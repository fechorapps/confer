using System.Reflection;
using Confer.Application;
using Confer.Domain.Identity;
using Confer.Infrastructure;
using Confer.Infrastructure.Persistence;
using Confer.Shared.Api;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

// Add Layers
builder.Services.AddApplication();
builder.Services.AddInfrastructure(builder.Configuration);

// Add Services
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.AllowAnyOrigin()
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

var app = builder.Build();

// Auto-migrate & seed test users
using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<ConferDbContext>();
    await db.Database.EnsureCreatedAsync();

    if (!await db.Users.AnyAsync())
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

app.UseCors();
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
