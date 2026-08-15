namespace Confer.Domain.Meetings;

public sealed record MeetingPolicy
{
    public bool AllowScreenShare { get; init; } = true;
    public bool AllowChat { get; init; } = true;
    public bool AllowUnmuteSelf { get; init; } = true;
    public bool MuteOnEntry { get; init; } = false;
    public bool AllowRename { get; init; } = true;

    public MeetingPolicy() { }

    public MeetingPolicy(
        bool allowScreenShare = true,
        bool allowChat = true,
        bool allowUnmuteSelf = true,
        bool muteOnEntry = false,
        bool allowRename = true)
    {
        AllowScreenShare = allowScreenShare;
        AllowChat = allowChat;
        AllowUnmuteSelf = allowUnmuteSelf;
        MuteOnEntry = muteOnEntry;
        AllowRename = allowRename;
    }

    public static MeetingPolicy Default => new();
}
