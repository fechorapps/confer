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
    public string RecordingMode { get; private set; } = "disabled";
    public bool IsRecording { get; private set; }
    public DateTime? RecordingStartedAt { get; private set; }
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? EndedAt { get; private set; }

    public ICollection<Session> Sessions { get; private set; } = new List<Session>();
    public ICollection<MeetingRecording> Recordings { get; private set; } = new List<MeetingRecording>();

    private Meeting() { }

    public static Result<Meeting> Create(
        string title,
        Guid ownerId,
        int maxParticipants = 50,
        string? customJoinCode = null)
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

        EndedAt = DateTime.UtcNow;
        return Result.Success();
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
