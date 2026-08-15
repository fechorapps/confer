using System.Net.Http.Json;
using System.Text.Json.Nodes;
using Confer.Application.Auth.Sso;
using Confer.Application.Meetings.Telephony;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Auth.Sso;
using Confer.Infrastructure.Telephony;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace Confer.Tests.SsoAndTelephony;

public sealed class SsoAndTelephonyTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly WebApplicationFactory<Program> _factory;
    private readonly HttpClient _client;

    public SsoAndTelephonyTests(WebApplicationFactory<Program> factory)
    {
        _factory = factory;
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task GetSsoProviders_ShouldReturnConfiguredEnterpriseProviders()
    {
        var response = await _client.GetAsync("/api/auth/sso/providers");
        Assert.True(response.IsSuccessStatusCode);

        var providers = await response.Content.ReadFromJsonAsync<List<SsoProviderDto>>();
        Assert.NotNull(providers);
        Assert.Contains(providers, p => p.Name == "google");
        Assert.Contains(providers, p => p.Name == "microsoft");
        Assert.Contains(providers, p => p.Name == "okta");
        Assert.Contains(providers, p => p.Name == "saml2");
    }

    [Fact]
    public async Task SsoAuthorize_WithGoogleProvider_ShouldReturnPkceChallenge()
    {
        var response = await _client.GetAsync("/api/auth/sso/google/authorize");
        Assert.True(response.IsSuccessStatusCode);

        var result = await response.Content.ReadFromJsonAsync<SsoAuthorizeResultDto>();
        Assert.NotNull(result);
        Assert.Contains("accounts.google.com", result.AuthorizationUrl);
        Assert.Contains("code_challenge=", result.AuthorizationUrl);
        Assert.NotEmpty(result.State);
        Assert.NotEmpty(result.CodeVerifier);
    }

    [Fact]
    public async Task SsoCallback_WithValidCode_ShouldProvisionUserAndIssueToken()
    {
        var callbackPayload = new
        {
            Code = "mock_auth_code_987654",
            State = "mock_state_123",
            CodeVerifier = "mock_verifier_xyz"
        };

        var response = await _client.PostAsJsonAsync("/api/auth/sso/okta/callback", callbackPayload);
        Assert.True(response.IsSuccessStatusCode);

        var result = await response.Content.ReadFromJsonAsync<SsoCallbackResultDto>();
        Assert.NotNull(result);
        Assert.NotEqual(Guid.Empty, result.UserId);
        Assert.Contains("okta", result.Email);
        Assert.NotEmpty(result.AccessToken);
    }

    [Fact]
    public async Task GetMeetingTelephony_WhenMeetingExists_ShouldReturnDialInInfo()
    {
        // 1. Create meeting via dev-login
        var loginRes = await _client.PostAsJsonAsync("/api/auth/dev-login", new { Email = "host_telephony@confer.local", DisplayName = "Telephony Host" });
        var loginData = await loginRes.Content.ReadFromJsonAsync<JsonObject>();
        var token = loginData!["token"]!.GetValue<string>();
        var userId = loginData!["userId"]!.GetValue<Guid>();

        _client.DefaultRequestHeaders.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);
        var createRes = await _client.PostAsJsonAsync("/api/meetings", new { Title = "Executive Strategy Call", OwnerId = userId, MaxParticipants = 25 });
        var meetingData = await createRes.Content.ReadFromJsonAsync<JsonObject>();
        var meetingId = meetingData!["id"]!.GetValue<Guid>();
        var joinCode = meetingData!["joinCode"]!.GetValue<string>();

        // 2. Query Telephony info
        var telRes = await _client.GetAsync($"/api/meetings/{meetingId}/telephony");
        Assert.True(telRes.IsSuccessStatusCode);

        var telInfo = await telRes.Content.ReadFromJsonAsync<PstnDialInInfoDto>();
        Assert.NotNull(telInfo);
        Assert.Equal(meetingId, telInfo.MeetingId);
        Assert.Equal(joinCode, telInfo.ConferencePin);
        Assert.Contains("sip:", telInfo.DirectSipUri);
        Assert.NotEmpty(telInfo.GlobalDialInNumbers);
    }

    [Fact]
    public async Task ProcessInboundSipCall_InitialGreeting_ShouldReturnGatherTwiML()
    {
        var formContent = new FormUrlEncodedContent(new Dictionary<string, string>
        {
            ["From"] = "+14155552671",
            ["CallSid"] = "CA123456789"
        });

        var response = await _client.PostAsync("/api/telephony/sip-inbound", formContent);
        Assert.True(response.IsSuccessStatusCode);

        var xml = await response.Content.ReadAsStringAsync();
        Assert.Contains("<Gather numDigits=\"6\"", xml);
        Assert.Contains("Welcome to Confer", xml);
    }

    [Fact]
    public async Task ProcessInboundSipCall_WithValidMeetingPin_ShouldBridgeIntoConference()
    {
        // 1. Create meeting
        var loginRes = await _client.PostAsJsonAsync("/api/auth/dev-login", new { Email = "dialin_host@confer.local", DisplayName = "DialIn Host" });
        var loginData = await loginRes.Content.ReadFromJsonAsync<JsonObject>();
        var token = loginData!["token"]!.GetValue<string>();
        var userId = loginData!["userId"]!.GetValue<Guid>();

        _client.DefaultRequestHeaders.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);
        var createRes = await _client.PostAsJsonAsync("/api/meetings", new { Title = "Budget Review", OwnerId = userId, MaxParticipants = 10 });
        var meetingData = await createRes.Content.ReadFromJsonAsync<JsonObject>();
        var joinCode = meetingData!["joinCode"]!.GetValue<string>();

        // 2. Simulate entering 6-digit PIN on phone keypad
        var formContent = new FormUrlEncodedContent(new Dictionary<string, string>
        {
            ["From"] = "+14155559988",
            ["Digits"] = joinCode,
            ["CallSid"] = "CA9988776655"
        });

        var response = await _client.PostAsync("/api/telephony/sip-inbound", formContent);
        Assert.True(response.IsSuccessStatusCode);

        var xml = await response.Content.ReadAsStringAsync();
        Assert.Contains("<Dial>", xml);
        Assert.Contains("<Conference", xml);
        Assert.Contains("Budget Review", xml);
    }
}
