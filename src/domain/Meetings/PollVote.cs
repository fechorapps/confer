namespace Confer.Domain.Meetings;

public sealed class PollVote
{
    public Guid Id { get; private set; }
    public Guid PollId { get; private set; }
    public Guid OptionId { get; private set; }
    public Guid VoterId { get; private set; }
    public DateTime VotedAt { get; private set; } = DateTime.UtcNow;

    private PollVote() { }

    public static PollVote Create(Guid pollId, Guid optionId, Guid voterId)
    {
        return new PollVote
        {
            Id = Guid.NewGuid(),
            PollId = pollId,
            OptionId = optionId,
            VoterId = voterId,
            VotedAt = DateTime.UtcNow
        };
    }
}
