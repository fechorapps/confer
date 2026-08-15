using Confer.Application.Meetings.Breakouts.CloseBreakouts;
using Confer.Application.Meetings.Breakouts.CreateBreakouts;
using Confer.Application.Meetings.Create;
using Confer.Application.Meetings.GetByCode;
using Confer.Application.Meetings.GetRecordings;
using Confer.Application.Meetings.Join;
using Confer.Application.Meetings.Lock;
using Confer.Application.Meetings.Polls.ClosePoll;
using Confer.Application.Meetings.Polls.CreatePoll;
using Confer.Application.Meetings.Polls.GetPolls;
using Confer.Application.Meetings.Polls.SubmitPollVote;
using Confer.Application.Meetings.StartRecording;
using Confer.Application.Meetings.StopRecording;
using Confer.Shared.Api;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Routing;

namespace Confer.Api.Endpoints.Meetings;

public class MeetingsEndpoint : BaseModule, IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        var group = app.MapGroup("/api/meetings")
            .WithTags("Meetings");

        group.MapPost("/", CreateMeeting).WithName("CreateMeeting");
        group.MapGet("/{code}", GetMeetingByCode).WithName("GetMeetingByCode");
        group.MapPost("/{id:guid}/join", JoinMeeting).WithName("JoinMeeting");
        group.MapPost("/{id:guid}/lock", LockMeeting).WithName("LockMeeting");
        group.MapPost("/{id:guid}/recording/start", StartRecording).WithName("StartRecording");
        group.MapPost("/{id:guid}/recordings/start", StartRecording).WithName("StartRecordingAlias");
        group.MapPost("/{id:guid}/recording/stop", StopRecording).WithName("StopRecording");
        group.MapPost("/{id:guid}/recordings/stop", StopRecording).WithName("StopRecordingAlias");
        group.MapGet("/{id:guid}/recordings", GetRecordings).WithName("GetRecordings");

        // Polls
        group.MapPost("/{id:guid}/polls", CreatePoll).WithName("CreatePoll");
        group.MapGet("/{id:guid}/polls", GetPolls).WithName("GetPolls");
        group.MapPost("/{id:guid}/polls/{pollId:guid}/vote", SubmitPollVote).WithName("SubmitPollVote");
        group.MapPost("/{id:guid}/polls/{pollId:guid}/close", ClosePoll).WithName("ClosePoll");

        // Breakout Rooms
        group.MapPost("/{id:guid}/breakouts", CreateBreakouts).WithName("CreateBreakouts");
        group.MapPost("/{id:guid}/breakouts/close", CloseBreakouts).WithName("CloseBreakouts");
    }

    public record CreateMeetingRequest(string Title, Guid OwnerId, int MaxParticipants = 50, string? CustomJoinCode = null);
    public record JoinMeetingRequest(Guid UserId, string DisplayName, string? ClientInfo = null);
    public record LockMeetingRequest(Guid ActorId, bool Lock);
    public record StartRecordingRequest(Guid ActorId);
    public record StopRecordingRequest(Guid ActorId);
    public record CreatePollRequest(Guid CreatorId, string Question, List<string> Options, bool IsAnonymous = false, bool IsMultiChoice = false);
    public record SubmitPollVoteRequest(Guid VoterId, List<Guid> OptionIds);
    public record ClosePollRequest(Guid ActorId);
    public record CreateBreakoutsRequest(Guid ActorId, int RoomCount, int MaxDurationMinutes = 15, List<string>? RoomNames = null, List<Confer.Application.DTOs.BreakoutAssignmentDto>? Assignments = null);
    public record CloseBreakoutsRequest(Guid ActorId);

    public record CreateMeetingResponse
    {
        public Guid Id { get; init; }
        public string JoinCode { get; init; } = string.Empty;
        public string Title { get; init; } = string.Empty;
        public Guid OwnerId { get; init; }
        public int MaxParticipants { get; init; }
        public DateTime CreatedAt { get; init; }
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
    }

    private static async Task<IResult> CreateMeeting(
        [FromBody] CreateMeetingRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CreateMeetingCommand(request.Title, request.OwnerId, request.MaxParticipants, request.CustomJoinCode);
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
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new LockMeetingCommand(id, request.ActorId, request.Lock);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }

    private static async Task<IResult> StartRecording(
        Guid id,
        [FromBody] StartRecordingRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new StartRecordingCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> StopRecording(
        Guid id,
        [FromBody] StopRecordingRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
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
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CreateBreakoutsCommand(id, request.ActorId, request.RoomCount, request.MaxDurationMinutes, request.RoomNames, request.Assignments);
        var result = await dispatcher.SendAsync(command, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> CloseBreakouts(
        Guid id,
        [FromBody] CloseBreakoutsRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CloseBreakoutsCommand(id, request.ActorId);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
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
