using Confer.Application.DTOs;
using Confer.Application.Meetings.Breakouts.CloseBreakouts;
using Confer.Application.Meetings.Breakouts.CreateBreakouts;
using Confer.Application.Meetings.Create;
using Confer.Application.Meetings.GetByCode;
using Confer.Application.Meetings.GetRecordings;
using Confer.Application.Meetings.Governance.AdmitAll;
using Confer.Application.Meetings.Governance.AdmitParticipant;
using Confer.Application.Meetings.Governance.GetPolicy;
using Confer.Application.Meetings.Governance.GetWaitingRoom;
using Confer.Application.Meetings.Governance.RejectParticipant;
using Confer.Application.Meetings.Governance.ToggleWaitingRoom;
using Confer.Application.Meetings.Governance.UpdatePolicy;
using Confer.Application.Meetings.Join;
using Confer.Application.Meetings.Lock;
using Confer.Application.Meetings.Polls.ClosePoll;
using Confer.Application.Meetings.Polls.CreatePoll;
using Confer.Application.Meetings.Polls.GetPolls;
using Confer.Application.Meetings.Polls.SubmitPollVote;
using Confer.Application.Meetings.StartRecording;
using Confer.Application.Meetings.StopRecording;
using Confer.Application.Meetings.Stream.GetLiveStreamStatus;
using Confer.Application.Meetings.Stream.StartLiveStream;
using Confer.Application.Meetings.Stream.StopLiveStream;
using Confer.Application.Interfaces;
using Confer.Application.Meetings.Summary;
using Confer.Domain.Meetings;
using Confer.Shared.Api;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Routing;
using Microsoft.EntityFrameworkCore;
using System.IdentityModel.Tokens.Jwt;

namespace Confer.Api.Endpoints.Meetings;

public class MeetingsEndpoint : BaseModule, IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        var group = app.MapGroup("/api/meetings")
            .WithTags("Meetings");

        // NOTE on authorization scope: this is the first security pass over this endpoint
        // group. Destructive/moderation actions (lock, recording, live-stream, waiting-room,
        // policies, breakouts) require a valid bearer token AND that the caller's own
        // identity (JWT `sub`) matches the ActorId/CreatorId asserted in the request body —
        // see EnsureActorAuthorized below. Create/join/read endpoints remain anonymous for
        // now to match how the web/desktop/mobile clients currently call them; broadening
        // authorization to those is tracked as a follow-up once those flows carry tokens too.
        group.MapPost("/", CreateMeeting).WithName("CreateMeeting");
        group.MapGet("/{code}", GetMeetingByCode).WithName("GetMeetingByCode");
        group.MapPost("/{id:guid}/join", JoinMeeting).WithName("JoinMeeting");
        group.MapPost("/{id:guid}/lock", LockMeeting).WithName("LockMeeting").RequireAuthorization();
        group.MapPost("/{id:guid}/recording/start", StartRecording).WithName("StartRecording").RequireAuthorization();
        group.MapPost("/{id:guid}/recordings/start", StartRecording).WithName("StartRecordingAlias").RequireAuthorization();
        group.MapPost("/{id:guid}/recording/stop", StopRecording).WithName("StopRecording").RequireAuthorization();
        group.MapPost("/{id:guid}/recordings/stop", StopRecording).WithName("StopRecordingAlias").RequireAuthorization();
        group.MapGet("/{id:guid}/recordings", GetRecordings).WithName("GetRecordings");

        // AI Summary & Action Items
        group.MapPost("/{id:guid}/summary", GenerateMeetingSummary).WithName("GenerateMeetingSummary");
        group.MapGet("/{id:guid}/summary", GetMeetingSummary).WithName("GetMeetingSummary");

        // Polls
        group.MapPost("/{id:guid}/polls", CreatePoll).WithName("CreatePoll");
        group.MapGet("/{id:guid}/polls", GetPolls).WithName("GetPolls");
        group.MapPost("/{id:guid}/polls/{pollId:guid}/vote", SubmitPollVote).WithName("SubmitPollVote");
        group.MapPost("/{id:guid}/polls/{pollId:guid}/close", ClosePoll).WithName("ClosePoll");

        // Breakout Rooms
        group.MapPost("/{id:guid}/breakouts", CreateBreakouts).WithName("CreateBreakouts").RequireAuthorization();
        group.MapPost("/{id:guid}/breakouts/close", CloseBreakouts).WithName("CloseBreakouts").RequireAuthorization();

        // Waiting Room & Governance
        group.MapPost("/{id:guid}/waiting-room/toggle", ToggleWaitingRoom).WithName("ToggleWaitingRoom").RequireAuthorization();
        group.MapPost("/{id:guid}/waiting-room/admit", AdmitParticipant).WithName("AdmitParticipant").RequireAuthorization();
        group.MapPost("/{id:guid}/waiting-room/admit-all", AdmitAllParticipants).WithName("AdmitAllParticipants").RequireAuthorization();
        group.MapPost("/{id:guid}/waiting-room/reject", RejectParticipant).WithName("RejectParticipant").RequireAuthorization();
        group.MapGet("/{id:guid}/waiting-room", GetWaitingRoom).WithName("GetWaitingRoom");
        group.MapPost("/{id:guid}/policies", UpdateMeetingPolicy).WithName("UpdateMeetingPolicy").RequireAuthorization();
        group.MapGet("/{id:guid}/policies", GetMeetingPolicy).WithName("GetMeetingPolicy");

        // Live Streaming & RTMP Broadcast
        group.MapPost("/{id:guid}/stream/start", StartLiveStream).WithName("StartLiveStream").RequireAuthorization();
        group.MapPost("/{id:guid}/stream/stop", StopLiveStream).WithName("StopLiveStream").RequireAuthorization();
        group.MapGet("/{id:guid}/stream", GetLiveStreamStatus).WithName("GetLiveStreamStatus");

        // iCalendar (.ics)
        group.MapGet("/{id:guid}/calendar.ics", GetMeetingCalendarIcs).WithName("GetMeetingCalendarIcs");
    }

    public record CreateMeetingRequest(
        string Title,
        Guid OwnerId,
        int MaxParticipants = 50,
        string? CustomJoinCode = null,
        bool IsWaitingRoomEnabled = false,
        bool IsWatermarkEnabled = false,
        MeetingPolicyDto? Policy = null
    );
    public record JoinMeetingRequest(Guid UserId, string DisplayName, string? ClientInfo = null);
    public record LockMeetingRequest(Guid ActorId, bool Lock);
    public record StartRecordingRequest(Guid ActorId);
    public record StopRecordingRequest(Guid ActorId);
    public record StartLiveStreamRequest(Guid ActorId, string RtmpUrl, string StreamKey);
    public record StopLiveStreamRequest(Guid ActorId);
    public record GenerateMeetingSummaryRequest(Guid ActorId, bool ForceRegenerate = false);
    public record CreatePollRequest(Guid CreatorId, string Question, List<string> Options, bool IsAnonymous = false, bool IsMultiChoice = false);
    public record SubmitPollVoteRequest(Guid VoterId, List<Guid> OptionIds);
    public record ClosePollRequest(Guid ActorId);
    public record CreateBreakoutsRequest(Guid ActorId, int RoomCount, int MaxDurationMinutes = 15, List<string>? RoomNames = null, List<Confer.Application.DTOs.BreakoutAssignmentDto>? Assignments = null);
    public record CloseBreakoutsRequest(Guid ActorId);
    public record ToggleWaitingRoomRequest(Guid ActorId, bool Enabled);
    public record AdmitParticipantRequest(Guid ActorId, Guid ParticipantId);
    public record AdmitAllRequest(Guid ActorId);
    public record RejectParticipantRequest(Guid ActorId, Guid ParticipantId);
    public record UpdateMeetingPolicyRequest(
        Guid ActorId,
        MeetingPolicyDto Policy,
        bool? IsWatermarkEnabled = null,
        bool? IsWaitingRoomEnabled = null
    );

    public record CreateMeetingResponse
    {
        public Guid Id { get; init; }
        public string JoinCode { get; init; } = string.Empty;
        public string Title { get; init; } = string.Empty;
        public Guid OwnerId { get; init; }
        public int MaxParticipants { get; init; }
        public DateTime CreatedAt { get; init; }
        public bool IsWaitingRoomEnabled { get; init; }
        public bool IsWatermarkEnabled { get; init; }
        public MeetingPolicyDto? Policy { get; init; }
    }

    public record JoinMeetingResponse
    {
        public Guid MeetingId { get; init; }
        public string Title { get; init; } = string.Empty;
        public Guid ParticipantId { get; init; }
        public string Role { get; init; } = string.Empty;
        public bool IsLocked { get; init; }
        public string WsUrl { get; init; } = string.Empty;
        public string RoomToken { get; init; } = string.Empty;
        public List<Confer.Application.DTOs.IceServerConfig> IceServers { get; init; } = new();
        public string Status { get; init; } = "admitted";
        public bool IsWaitingRoom { get; init; }
    }

    private static async Task<IResult> CreateMeeting(
        [FromBody] CreateMeetingRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CreateMeetingCommand(
            request.Title,
            request.OwnerId,
            request.MaxParticipants,
            request.CustomJoinCode,
            request.IsWaitingRoomEnabled,
            request.IsWatermarkEnabled,
            request.Policy);
        var result = await dispatcher.SendAsync(command, ct);
        return result.IsSuccess
            ? TypedResults.Created($"/api/meetings/{result.Value.JoinCode}", result.Value)
            : HandleFailure(result.Error);
    }

    private static async Task<IResult> GetMeetingByCode(
        string code,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetMeetingByCodeQuery(code);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> JoinMeeting(
        Guid id,
        [FromBody] JoinMeetingRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new JoinMeetingCommand(id, request.UserId, request.DisplayName, request.ClientInfo);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> LockMeeting(
        Guid id,
        [FromBody] LockMeetingRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new LockMeetingCommand(id, request.ActorId, request.Lock);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }

    private static async Task<IResult> StartRecording(
        Guid id,
        [FromBody] StartRecordingRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new StartRecordingCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> StopRecording(
        Guid id,
        [FromBody] StopRecordingRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new StopRecordingCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GetRecordings(
        Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetMeetingRecordingsQuery(id);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GenerateMeetingSummary(
        Guid id,
        [FromBody] GenerateMeetingSummaryRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new GenerateMeetingSummaryCommand(id, request.ActorId, request.ForceRegenerate);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GetMeetingSummary(
        Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetMeetingSummaryQuery(id);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> CreatePoll(
        Guid id,
        [FromBody] CreatePollRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CreatePollCommand(id, request.CreatorId, request.Question, request.Options, request.IsAnonymous, request.IsMultiChoice);
        var result = await dispatcher.SendAsync(command, ct);
        return result.IsSuccess
            ? TypedResults.Created($"/api/meetings/{id}/polls/{result.Value.Id}", result.Value)
            : HandleFailure(result.Error);
    }

    private static async Task<IResult> GetPolls(
        Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetPollsQuery(id);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> SubmitPollVote(
        Guid id,
        Guid pollId,
        [FromBody] SubmitPollVoteRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new SubmitPollVoteCommand(id, pollId, request.VoterId, request.OptionIds);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> ClosePoll(
        Guid id,
        Guid pollId,
        [FromBody] ClosePollRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new ClosePollCommand(id, pollId, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> CreateBreakouts(
        Guid id,
        [FromBody] CreateBreakoutsRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new CreateBreakoutsCommand(id, request.ActorId, request.RoomCount, request.MaxDurationMinutes, request.RoomNames, request.Assignments);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> CloseBreakouts(
        Guid id,
        [FromBody] CloseBreakoutsRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new CloseBreakoutsCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }

    private static async Task<IResult> ToggleWaitingRoom(
        Guid id,
        [FromBody] ToggleWaitingRoomRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new ToggleWaitingRoomCommand(id, request.ActorId, request.Enabled);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> AdmitParticipant(
        Guid id,
        [FromBody] AdmitParticipantRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new AdmitParticipantCommand(id, request.ActorId, request.ParticipantId);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }

    private static async Task<IResult> AdmitAllParticipants(
        Guid id,
        [FromBody] AdmitAllRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new AdmitAllCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> RejectParticipant(
        Guid id,
        [FromBody] RejectParticipantRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new RejectParticipantCommand(id, request.ActorId, request.ParticipantId);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }

    private static async Task<IResult> GetWaitingRoom(
        Guid id,
        [FromQuery] Guid? actorId,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetWaitingRoomQuery(id, actorId);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> UpdateMeetingPolicy(
        Guid id,
        [FromBody] UpdateMeetingPolicyRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new UpdateMeetingPolicyCommand(id, request.ActorId, request.Policy, request.IsWatermarkEnabled, request.IsWaitingRoomEnabled);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GetMeetingPolicy(
        Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetMeetingPolicyQuery(id);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> StartLiveStream(
        Guid id,
        [FromBody] StartLiveStreamRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new StartLiveStreamCommand(id, request.ActorId, request.RtmpUrl, request.StreamKey);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> StopLiveStream(
        Guid id,
        [FromBody] StopLiveStreamRequest request,
        HttpContext httpContext,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        if (EnsureActorAuthorized(httpContext, request.ActorId) is { } forbidden) return forbidden;

        var command = new StopLiveStreamCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GetLiveStreamStatus(
        Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new GetLiveStreamStatusQuery(id);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> GetMeetingCalendarIcs(
        [FromRoute] Guid id,
        [FromServices] IConferDbContext dbContext,
        [FromServices] ICalendarService calendarService,
        HttpContext httpContext,
        CancellationToken ct)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == id, ct);

        if (meeting is null)
        {
            return TypedResults.NotFound(new { Code = "Meeting.NotFound", Description = "The meeting was not found." });
        }

        var organizer = await dbContext.Users
            .FirstOrDefaultAsync(u => u.Id == meeting.OwnerId, ct);

        var baseUrl = $"{httpContext.Request.Scheme}://{httpContext.Request.Host}";
        var icsContent = calendarService.GenerateMeetingIcs(meeting, organizer, baseUrl);

        return Results.Content(icsContent, "text/calendar; charset=utf-8");
    }

    /// <summary>
    /// Verifies the authenticated caller (JWT `sub` claim) is the same identity as the
    /// ActorId/CreatorId asserted in the request body, so a valid token for user A cannot be
    /// used to moderate a meeting while claiming to be user B. Returns a 403 IResult to short-
    /// circuit the handler on mismatch, or null when the caller is who they claim to be.
    /// </summary>
    private static IResult? EnsureActorAuthorized(HttpContext httpContext, Guid actorId)
    {
        var sub = httpContext.User.FindFirst(JwtRegisteredClaimNames.Sub)?.Value;
        if (!Guid.TryParse(sub, out var callerId) || callerId != actorId)
        {
            return TypedResults.Forbid();
        }

        return null;
    }

    private static IResult HandleFailure(Confer.Shared.Results.Error error) =>
        error.Type switch
        {
            Confer.Shared.Results.ErrorType.NotFound => TypedResults.NotFound(new { error.Code, error.Description }),
            Confer.Shared.Results.ErrorType.Validation => TypedResults.BadRequest(new { error.Code, error.Description }),
            Confer.Shared.Results.ErrorType.Conflict => TypedResults.Conflict(new { error.Code, error.Description }),
            Confer.Shared.Results.ErrorType.Unauthorized => TypedResults.Unauthorized(),
            Confer.Shared.Results.ErrorType.Forbidden => TypedResults.Forbid(),
            _ => TypedResults.BadRequest(new { error.Code, error.Description })
        };
}
