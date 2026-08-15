using System.Net;
using System.Net.Http.Json;
using Confer.Api.Endpoints.Meetings;
using Confer.Api.Endpoints.Webhooks;
using Confer.Application.Meetings.Create;
using Confer.Application.Webhooks;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class WebhooksAndCalendarIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public WebhooksAndCalendarIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task WebhookSubscriptions_FullLifecycle_ShouldSucceed()
    {
        var userId = Guid.NewGuid();
        var targetUrl = $"https://integration-test.example.com/hooks/{Guid.NewGuid()}";

        // 1. Create Webhook Subscription
        var createReq = new WebhooksEndpoint.CreateWebhookSubscriptionRequest(
            userId,
            targetUrl,
            new List<string> { "meeting.created", "meeting.ended" }
        );

        var createResp = await _client.PostAsJsonAsync("/api/webhooks", createReq);
        createResp.StatusCode.Should().Be(HttpStatusCode.Created);

        var created = await createResp.Content.ReadFromJsonAsync<WebhookSubscriptionDto>();
        created.Should().NotBeNull();
        created!.TargetUrl.Should().Be(targetUrl);
        created.SubscribedEvents.Should().Contain("meeting.created");
        created.Secret.Should().StartWith("whsec_");

        // 2. Duplicate URL for same user should return 409 Conflict
        var dupResp = await _client.PostAsJsonAsync("/api/webhooks", createReq);
        dupResp.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 3. Invalid URL should return 400 Bad Request
        var invalidReq = new WebhooksEndpoint.CreateWebhookSubscriptionRequest(
            userId,
            "ftp://invalid-url.com",
            new List<string> { "meeting.created" }
        );
        var invalidResp = await _client.PostAsJsonAsync("/api/webhooks", invalidReq);
        invalidResp.StatusCode.Should().Be(HttpStatusCode.BadRequest);

        // 4. List Subscriptions filtered by user
        var listResp = await _client.GetAsync($"/api/webhooks?userId={userId}");
        listResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var list = await listResp.Content.ReadFromJsonAsync<List<WebhookSubscriptionDto>>();
        list.Should().NotBeNull();
        list!.Should().ContainSingle(w => w.Id == created.Id);

        // 5. Delete Subscription
        var deleteResp = await _client.DeleteAsync($"/api/webhooks/{created.Id}?userId={userId}");
        deleteResp.StatusCode.Should().Be(HttpStatusCode.NoContent);

        // 6. Delete again should return 404
        var deleteAgainResp = await _client.DeleteAsync($"/api/webhooks/{created.Id}");
        deleteAgainResp.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }

    [Fact]
    public async Task GetMeetingCalendarIcs_WhenMeetingExists_ShouldReturnCalendarContent()
    {
        // 1. Create a meeting
        var ownerId = Guid.NewGuid();
        var createMeetingReq = new MeetingsEndpoint.CreateMeetingRequest(
            Title: "Release Planning",
            OwnerId: ownerId,
            MaxParticipants: 30,
            CustomJoinCode: $"rel-{Guid.NewGuid().ToString()[..6]}"
        );

        var createMeetingResp = await _client.PostAsJsonAsync("/api/meetings", createMeetingReq);
        createMeetingResp.StatusCode.Should().Be(HttpStatusCode.Created);
        var createdMeeting = await createMeetingResp.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        createdMeeting.Should().NotBeNull();

        // 2. Query .ics calendar endpoint
        var icsResp = await _client.GetAsync($"/api/meetings/{createdMeeting!.Id}/calendar.ics");
        icsResp.StatusCode.Should().Be(HttpStatusCode.OK);
        icsResp.Content.Headers.ContentType?.MediaType.Should().Be("text/calendar");

        var icsContent = await icsResp.Content.ReadAsStringAsync();
        icsContent.Should().Contain("BEGIN:VCALENDAR");
        icsContent.Should().Contain("BEGIN:VEVENT");
        icsContent.Should().Contain("SUMMARY:Release Planning");
        icsContent.Should().Contain(createdMeeting.JoinCode);
        icsContent.Should().Contain("BEGIN:VALARM");
        icsContent.Should().Contain("TRIGGER:-PT15M");
        icsContent.Should().Contain("END:VALARM");
        icsContent.Should().Contain("END:VEVENT");
        icsContent.Should().Contain("END:VCALENDAR");
    }

    [Fact]
    public async Task GetMeetingCalendarIcs_WhenMeetingNotFound_ShouldReturn404()
    {
        var nonExistentId = Guid.NewGuid();
        var icsResp = await _client.GetAsync($"/api/meetings/{nonExistentId}/calendar.ics");
        icsResp.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }
}
