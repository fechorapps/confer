using System.Reflection;
using System.Text;
using System.Threading.RateLimiting;
using Confer.Application;
using Confer.Domain.Identity;
using Confer.Infrastructure;
using Confer.Infrastructure.Persistence;
using Confer.Shared.Api;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.RateLimiting;
using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Tokens;

var builder = WebApplication.CreateBuilder(args);

// Add Layers
builder.Services.AddApplication();
builder.Services.AddInfrastructure(builder.Configuration);

// Add Services
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();

// JWT authentication. The default secret below matches JwtTokenProvider's dev fallback so
// local development keeps working without extra setup; Production refuses to boot on it.
const string DevOnlyJwtSecretFallback = "confer-super-secret-key-that-is-at-least-256-bits-long-for-hmac-sha256!";
var jwtSecret = builder.Configuration["Jwt:Secret"];
var jwtIssuer = builder.Configuration["Jwt:Issuer"] ?? "confer-api";
var jwtAudience = builder.Configuration["Jwt:Audience"] ?? "confer-client";

if (builder.Environment.IsProduction() && string.IsNullOrWhiteSpace(jwtSecret))
{
    throw new InvalidOperationException(
        "Jwt:Secret must be configured (e.g. via the Jwt__Secret environment variable or a secret store) before running in Production. " +
        "Refusing to start with the built-in development fallback secret.");
}

jwtSecret ??= DevOnlyJwtSecretFallback;

builder.Services
    .AddAuthentication(JwtBearerDefaults.AuthenticationScheme)
    .AddJwtBearer(options =>
    {
        // Keep claim types exactly as issued by JwtTokenProvider ("sub", "email", "name")
        // instead of the legacy ClaimTypes.* remapping ASP.NET applies by default.
        options.MapInboundClaims = false;
        options.TokenValidationParameters = new TokenValidationParameters
        {
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(jwtSecret)),
            ValidateIssuer = true,
            ValidIssuer = jwtIssuer,
            ValidateAudience = true,
            ValidAudience = jwtAudience,
            ClockSkew = TimeSpan.FromSeconds(30),
        };
    });

builder.Services.AddAuthorization();

// CORS: explicit allow-list instead of AllowAnyOrigin. Configure via Cors:AllowedOrigins in
// appsettings/environment; falls back to the bundled web client's own origin for local dev.
var allowedOrigins = builder.Configuration.GetSection("Cors:AllowedOrigins").Get<string[]>()
    ?? ["http://localhost:5000", "https://localhost:5001"];

builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.WithOrigins(allowedOrigins)
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

// Rate limiting for the login and meeting-creation endpoints, partitioned per client IP.
builder.Services.AddRateLimiter(options =>
{
    options.RejectionStatusCode = StatusCodes.Status429TooManyRequests;

    static string PartitionKey(HttpContext ctx) => ctx.Connection.RemoteIpAddress?.ToString() ?? "unknown";

    options.AddPolicy("auth", ctx => RateLimitPartition.GetFixedWindowLimiter(
        PartitionKey(ctx),
        _ => new FixedWindowRateLimiterOptions { PermitLimit = 20, Window = TimeSpan.FromMinutes(1), QueueLimit = 0 }));

    options.AddPolicy("meetings-write", ctx => RateLimitPartition.GetFixedWindowLimiter(
        PartitionKey(ctx),
        _ => new FixedWindowRateLimiterOptions { PermitLimit = 30, Window = TimeSpan.FromMinutes(1), QueueLimit = 0 }));
});

var app = builder.Build();

// Apply pending EF Core migrations. Replaces the previous EnsureCreatedAsync() + hand-rolled
// ALTER TABLE statements wrapped in empty try/catch blocks, which silently swallowed real
// schema failures and had no way to express anything beyond "add this column if missing".
using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<ConferDbContext>();
    await db.Database.MigrateAsync();

    // Seed demo accounts only in Development — Production/Staging should never carry
    // well-known, publicly documented user records.
    if (app.Environment.IsDevelopment() && !await db.Users.AnyAsync())
    {
        db.Users.AddRange(
            User.CreateWithId(Guid.Parse("11111111-1111-1111-1111-111111111111"), "host@confer.local", "Host User (Dev)"),
            User.CreateWithId(Guid.Parse("22222222-2222-2222-2222-222222222222"), "participant1@confer.local", "Alice (Dev)"),
            User.CreateWithId(Guid.Parse("33333333-3333-3333-3333-333333333333"), "participant2@confer.local", "Bob (Dev)")
        );
        await db.SaveChangesAsync();
    }
}

if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI();
}

app.UseDefaultFiles();
app.UseStaticFiles();

app.UseCors();
app.UseAuthentication();
app.UseAuthorization();
app.UseRateLimiter();

app.UseWebSockets(new WebSocketOptions
{
    KeepAliveInterval = TimeSpan.FromSeconds(15)
});

// Health probes
app.MapGet("/v1/health", () => TypedResults.Ok(new { status = "healthy", timestamp = DateTime.UtcNow }));

// Auto-map all IEndpoint implementations
var endpointTypes = Assembly.GetExecutingAssembly().GetTypes()
    .Where(t => typeof(IEndpoint).IsAssignableFrom(t) && !t.IsInterface && !t.IsAbstract);

foreach (var type in endpointTypes)
{
    if (Activator.CreateInstance(type) is IEndpoint endpoint)
    {
        endpoint.MapEndpoint(app);
    }
}

app.Run();

// Required for WebApplicationFactory in integration tests
public partial class Program { }
