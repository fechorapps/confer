namespace Confer.Domain.Meetings;

public sealed class PollOption
{
    public Guid Id { get; private set; }
    public Guid PollId { get; private set; }
    public string Text { get; private set; } = string.Empty;
    public int Index { get; private set; }
    public int VoteCount { get; private set; }

    private PollOption() { }

    public static PollOption Create(Guid pollId, string text, int index)
    {
        return new PollOption
        {
            Id = Guid.NewGuid(),
            PollId = pollId,
            Text = text.Trim(),
            Index = index,
            VoteCount = 0
        };
    }

    public void IncrementVote(int count = 1)
    {
        VoteCount += count;
    }
}
