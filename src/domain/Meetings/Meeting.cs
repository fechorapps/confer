using Confer.Domain.Enums;
using Confer.Domain.Sessions;
using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public sealed class Meeting
{
    public Guid Id { get; private set; }
    public string JoinCode { get; private set; } = string.Empty;
    public string Title { get; private set; } = string.Empty;
    public Guid OwnerId { get; private set; }
    public DateTime? ScheduledStart { get; private set; }
    public int MaxParticipants { get; private set; } = 50;
    public bool IsLocked { get; private set; }
    public bool IsWaitingRoomEnabled { get; private set; }
    public bool IsWatermarkEnabled { get; private set; }
    public MeetingPolicy Policy { get; private set; } = MeetingPolicy.Default;
    public string RecordingMode { get; private set; } = "disabled";
    public bool IsRecording { get; private set; }
    public DateTime? RecordingStartedAt { get; private set; }
    public LiveStreamConfig LiveStreamConfig { get; private set; } = LiveStreamConfig.Default;
    public bool IsLiveStreaming => LiveStreamConfig.Status == LiveStreamStatus.Live;
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? EndedAt { get; private set; }

    public ICollection<Session> Sessions { get; private set; } = new List<Session>();
    public ICollection<MeetingRecording> Recordings { get; private set; } = new List<MeetingRecording>();
    public ICollection<Poll> Polls { get; private set; } = new List<Poll>();
    public ICollection<BreakoutRoom> BreakoutRooms { get; private set; } = new List<BreakoutRoom>();
    public MeetingSummary? Summary { get; private set; }

    private Meeting() { }

    public static Result<Meeting> Create(
        string title,
        Guid ownerId,
        int maxParticipants = 50,
        string? customJoinCode = null,
        bool isWaitingRoomEnabled = false,
        bool isWatermarkEnabled = false,
        MeetingPolicy? policy = null)
    {
        if (string.IsNullOrWhiteSpace(title))
            return Result.Failure<Meeting>(Error.Validation("Meeting.TitleEmpty", "Meeting title cannot be empty."));

        if (maxParticipants is < 2 or > 200)
            return Result.Failure<Meeting>(Error.Validation("Meeting.InvalidCapacity", "Max participants must be between 2 and 200."));

        var joinCode = !string.IsNullOrWhiteSpace(customJoinCode)
            ? customJoinCode.Trim().ToLowerInvariant()
            : GenerateJoinCode();

        var meeting = new Meeting
        {
            Id = Guid.NewGuid(),
            Title = title.Trim(),
            JoinCode = joinCode,
            OwnerId = ownerId,
            MaxParticipants = maxParticipants,
            IsLocked = false,
            IsWaitingRoomEnabled = isWaitingRoomEnabled,
            IsWatermarkEnabled = isWatermarkEnabled,
            Policy = policy ?? MeetingPolicy.Default,
            CreatedAt = DateTime.UtcNow
        };

        return Result.Success(meeting);
    }

    public Result CanJoin(int currentParticipantCount)
    {
        if (EndedAt.HasValue)
            return Result.Failure(MeetingErrors.Ended);

        if (IsLocked)
            return Result.Failure(MeetingErrors.Locked);

        if (currentParticipantCount >= MaxParticipants)
            return Result.Failure(MeetingErrors.Full);

        return Result.Success();
    }

    public Result Lock(Guid actorId)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        IsLocked = true;
        return Result.Success();
    }

    public Result Unlock(Guid actorId)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        IsLocked = false;
        return Result.Success();
    }

    public Result ToggleWaitingRoom(Guid actorId, bool enabled)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        IsWaitingRoomEnabled = enabled;
        return Result.Success();
    }

    public Result ToggleWatermark(Guid actorId, bool enabled)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        IsWatermarkEnabled = enabled;
        return Result.Success();
    }

    public Result UpdatePolicy(Guid actorId, MeetingPolicy policy)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        if (policy is null)
            return Result.Failure(MeetingErrors.InvalidPolicy);

        Policy = policy;
        return Result.Success();
    }

    public Result<Participation> AdmitParticipant(Guid actorId, Guid participantId)
    {
        if (actorId != OwnerId)
            return Result.Failure<Participation>(MeetingErrors.Unauthorized);

        var activeSession = Sessions.FirstOrDefault(s => s.EndedAt == null);
        if (activeSession is null)
            return Result.Failure<Participation>(SessionErrors.ParticipantNotFound);

        var participation = activeSession.Participations.FirstOrDefault(p => p.Id == participantId);
        if (participation is null)
            return Result.Failure<Participation>(SessionErrors.ParticipantNotFound);

        if (participation.Status != ParticipationStatus.InWaitingRoom)
            return Result.Failure<Participation>(MeetingErrors.ParticipantNotInWaitingRoom);

        participation.Admit();
        return Result.Success(participation);
    }

    public Result<List<Participation>> AdmitAll(Guid actorId)
    {
        if (actorId != OwnerId)
            return Result.Failure<List<Participation>>(MeetingErrors.Unauthorized);

        var activeSession = Sessions.FirstOrDefault(s => s.EndedAt == null);
        if (activeSession is null)
            return Result.Success(new List<Participation>());

        var waiting = activeSession.Participations
            .Where(p => p.Status == ParticipationStatus.InWaitingRoom && p.LeftAt == null)
            .ToList();

        foreach (var p in waiting)
        {
            p.Admit();
        }

        return Result.Success(waiting);
    }

    public Result<Participation> RejectParticipant(Guid actorId, Guid participantId)
    {
        if (actorId != OwnerId)
            return Result.Failure<Participation>(MeetingErrors.Unauthorized);

        var activeSession = Sessions.FirstOrDefault(s => s.EndedAt == null);
        if (activeSession is null)
            return Result.Failure<Participation>(SessionErrors.ParticipantNotFound);

        var participation = activeSession.Participations.FirstOrDefault(p => p.Id == participantId);
        if (participation is null)
            return Result.Failure<Participation>(SessionErrors.ParticipantNotFound);

        if (participation.Status != ParticipationStatus.InWaitingRoom)
            return Result.Failure<Participation>(MeetingErrors.ParticipantNotInWaitingRoom);

        participation.Reject();
        return Result.Success(participation);
    }

    public Result<MeetingRecording> StartRecording(Guid actorId)
    {
        if (EndedAt.HasValue)
            return Result.Failure<MeetingRecording>(MeetingErrors.Ended);

        if (actorId != OwnerId)
            return Result.Failure<MeetingRecording>(MeetingErrors.Unauthorized);

        if (IsRecording)
            return Result.Failure<MeetingRecording>(MeetingErrors.AlreadyRecording);

        IsRecording = true;
        RecordingStartedAt = DateTime.UtcNow;

        var recording = MeetingRecording.Create(Id, actorId);
        Recordings.Add(recording);
        return Result.Success(recording);
    }

    public Result<MeetingRecording> StopRecording(Guid actorId, string? storagePath = null, long fileSizeBytes = 0)
    {
        if (actorId != OwnerId)
            return Result.Failure<MeetingRecording>(MeetingErrors.Unauthorized);

        if (!IsRecording)
            return Result.Failure<MeetingRecording>(MeetingErrors.NotRecording);

        IsRecording = false;

        var activeRecording = Recordings.LastOrDefault(r => r.Status == Enums.RecordingStatus.Recording);
        if (activeRecording is null)
        {
            activeRecording = MeetingRecording.Create(Id, actorId);
            Recordings.Add(activeRecording);
        }

        activeRecording.Complete(storagePath ?? string.Empty, fileSizeBytes);
        return Result.Success(activeRecording);
    }

    public Result<LiveStreamConfig> StartLiveStream(Guid actorId, string rtmpUrl, string streamKey)
    {
        if (EndedAt.HasValue)
            return Result.Failure<LiveStreamConfig>(MeetingErrors.Ended);

        if (actorId != OwnerId)
            return Result.Failure<LiveStreamConfig>(MeetingErrors.Unauthorized);

        if (string.IsNullOrWhiteSpace(rtmpUrl))
            return Result.Failure<LiveStreamConfig>(Error.Validation("LiveStream.InvalidRtmpUrl", "RTMP URL cannot be empty."));

        if (string.IsNullOrWhiteSpace(streamKey))
            return Result.Failure<LiveStreamConfig>(Error.Validation("LiveStream.InvalidStreamKey", "Stream key cannot be empty."));

        if (LiveStreamConfig.Status == LiveStreamStatus.Live)
            return Result.Failure<LiveStreamConfig>(MeetingErrors.AlreadyStreaming);

        LiveStreamConfig = new LiveStreamConfig(
            isStreamingEnabled: true,
            rtmpUrl: rtmpUrl.Trim(),
            streamKey: streamKey.Trim(),
            status: LiveStreamStatus.Live,
            startedAt: DateTime.UtcNow
        );

        return Result.Success(LiveStreamConfig);
    }

    public Result<LiveStreamConfig> StopLiveStream(Guid actorId)
    {
        if (actorId != OwnerId)
            return Result.Failure<LiveStreamConfig>(MeetingErrors.Unauthorized);

        if (LiveStreamConfig.Status != LiveStreamStatus.Live)
            return Result.Failure<LiveStreamConfig>(MeetingErrors.NotStreaming);

        LiveStreamConfig = new LiveStreamConfig(
            isStreamingEnabled: false,
            rtmpUrl: LiveStreamConfig.RtmpUrl,
            streamKey: string.Empty,
            status: LiveStreamStatus.Idle,
            startedAt: null
        );

        return Result.Success(LiveStreamConfig);
    }

    public Result End(Guid actorId)
    {
        if (actorId != OwnerId)
            return Result.Failure(MeetingErrors.Unauthorized);

        if (IsRecording)
        {
            IsRecording = false;
            var activeRecording = Recordings.LastOrDefault(r => r.Status == Enums.RecordingStatus.Recording);
            activeRecording?.Complete(string.Empty, 0);
        }

        if (LiveStreamConfig.Status == LiveStreamStatus.Live)
        {
            LiveStreamConfig = new LiveStreamConfig(
                isStreamingEnabled: false,
                rtmpUrl: LiveStreamConfig.RtmpUrl,
                streamKey: string.Empty,
                status: LiveStreamStatus.Idle,
                startedAt: null
            );
        }

        EndedAt = DateTime.UtcNow;
        return Result.Success();
    }

    public Result<MeetingSummary> AddSummary(
        string overview,
        List<string>? keyDecisions = null,
        List<ActionItem>? actionItems = null,
        int durationMinutes = 0,
        int participantCount = 0)
    {
        if (!EndedAt.HasValue)
            return Result.Failure<MeetingSummary>(MeetingErrors.MeetingNotEnded);

        var summaryResult = MeetingSummary.Create(
            Id,
            overview,
            keyDecisions,
            actionItems,
            durationMinutes,
            participantCount);

        if (summaryResult.IsFailure)
            return summaryResult;

        Summary = summaryResult.Value;
        return Result.Success(Summary);
    }

    public Result<MeetingSummary> AddSummary(MeetingSummary summary)
    {
        if (summary is null)
            return Result.Failure<MeetingSummary>(Error.Validation("MeetingSummary.Null", "Summary cannot be null."));

        if (!EndedAt.HasValue)
            return Result.Failure<MeetingSummary>(MeetingErrors.MeetingNotEnded);

        Summary = summary;
        return Result.Success(summary);
    }

    private static string GenerateJoinCode()
    {
        const string chars = "abcdefghijklmnopqrstuvwxyz0123456789";
        var random = Random.Shared;
        return $"{new string(Enumerable.Range(0, 3).Select(_ => chars[random.Next(chars.Length)]).ToArray())}-" +
               $"{new string(Enumerable.Range(0, 4).Select(_ => chars[random.Next(chars.Length)]).ToArray())}-" +
               $"{new string(Enumerable.Range(0, 3).Select(_ => chars[random.Next(chars.Length)]).ToArray())}";
    }
}
