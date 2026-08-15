namespace Confer.Domain.Sessions;

public sealed class CaptionChunk
{
    public Guid Id { get; private set; }
    public Guid SessionId { get; private set; }
    public Guid ParticipantId { get; private set; }
    public string SpeakerName { get; private set; } = string.Empty;
    public string Text { get; private set; } = string.Empty;
    public string Language { get; private set; } = "en-US";
    public DateTime SpokenAt { get; private set; } = DateTime.UtcNow;

    private CaptionChunk() { }

    public static CaptionChunk Create(Guid sessionId, Guid participantId, string speakerName, string text, string language)
    {
        return new CaptionChunk
        {
            Id = Guid.NewGuid(),
            SessionId = sessionId,
            ParticipantId = participantId,
            SpeakerName = speakerName,
            Text = text,
            Language = language,
            SpokenAt = DateTime.UtcNow
        };
    }
}
