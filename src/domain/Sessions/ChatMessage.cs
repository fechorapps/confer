namespace Confer.Domain.Sessions;

public sealed class ChatMessage
{
    public Guid Id { get; private set; }
    public Guid SessionId { get; private set; }
    public Guid UserId { get; private set; }
    public string UserName { get; private set; } = string.Empty;
    public string Body { get; private set; } = string.Empty;
    public DateTime SentAt { get; private set; } = DateTime.UtcNow;

    private ChatMessage() { }

    public static ChatMessage Create(Guid sessionId, Guid userId, string userName, string body)
    {
        return new ChatMessage
        {
            Id = Guid.NewGuid(),
            SessionId = sessionId,
            UserId = userId,
            UserName = userName,
            Body = body,
            SentAt = DateTime.UtcNow
        };
    }
}
