namespace Confer.Domain.Enums;

public enum ParticipantRole
{
    Host = 1,
    CoHost = 2,
    Participant = 3
}

public enum ParticipationStatus
{
    Admitted = 1,
    InWaitingRoom = 2,
    Rejected = 3
}

public enum MediaKind
{
    Audio = 1,
    Video = 2,
    ScreenShare = 3
}

public enum SimulcastLayer
{
    F = 1, // Full (1280x720 @ 30fps)
    H = 2, // Half (640x360 @ 30fps)
    Q = 3  // Quarter (320x180 @ 15fps)
}

public enum LeaveReason
{
    Disconnected = 1,
    UserLeft = 2,
    Kicked = 3,
    MeetingEnded = 4
}

public enum RecordingStatus
{
    Stopped = 0,
    Recording = 1,
    Paused = 2,
    Processing = 3,
    Completed = 4,
    Failed = 5
}

public enum LiveStreamStatus
{
    Idle = 0,
    Live = 1,
    Failed = 2
}
