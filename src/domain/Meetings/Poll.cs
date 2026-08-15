using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public sealed class Poll
{
    public Guid Id { get; private set; }
    public Guid MeetingId { get; private set; }
    public Guid CreatorId { get; private set; }
    public string Question { get; private set; } = string.Empty;
    public bool IsAnonymous { get; private set; }
    public bool IsMultiChoice { get; private set; }
    public bool IsActive { get; private set; } = true;
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? ClosedAt { get; private set; }

    public ICollection<PollOption> Options { get; private set; } = new List<PollOption>();
    public ICollection<PollVote> Votes { get; private set; } = new List<PollVote>();

    private Poll() { }

    public static Result<Poll> Create(
        Guid meetingId,
        Guid creatorId,
        string question,
        List<string> optionTexts,
        bool isAnonymous = false,
        bool isMultiChoice = false)
    {
        if (meetingId == Guid.Empty)
            return Result.Failure<Poll>(Error.Validation("Poll.MeetingIdEmpty", "Meeting ID cannot be empty."));

        if (creatorId == Guid.Empty)
            return Result.Failure<Poll>(Error.Validation("Poll.CreatorIdEmpty", "Creator ID cannot be empty."));

        if (string.IsNullOrWhiteSpace(question))
            return Result.Failure<Poll>(Error.Validation("Poll.QuestionEmpty", "Poll question cannot be empty."));

        var validOptions = optionTexts?
            .Where(t => !string.IsNullOrWhiteSpace(t))
            .Select(t => t.Trim())
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToList() ?? new List<string>();

        if (validOptions.Count < 2)
            return Result.Failure<Poll>(PollErrors.InvalidOptions);

        var pollId = Guid.NewGuid();
        var poll = new Poll
        {
            Id = pollId,
            MeetingId = meetingId,
            CreatorId = creatorId,
            Question = question.Trim(),
            IsAnonymous = isAnonymous,
            IsMultiChoice = isMultiChoice,
            IsActive = true,
            CreatedAt = DateTime.UtcNow
        };

        for (int i = 0; i < validOptions.Count; i++)
        {
            poll.Options.Add(PollOption.Create(pollId, validOptions[i], i));
        }

        return Result.Success(poll);
    }

    public Result<List<PollVote>> Vote(Guid voterId, List<Guid> optionIds)
    {
        if (voterId == Guid.Empty)
            return Result.Failure<List<PollVote>>(Error.Validation("Poll.VoterIdEmpty", "Voter ID cannot be empty."));

        if (!IsActive)
            return Result.Failure<List<PollVote>>(PollErrors.Closed);

        if (optionIds == null || optionIds.Count == 0)
            return Result.Failure<List<PollVote>>(PollErrors.OptionNotFound);

        if (!IsMultiChoice && optionIds.Count > 1)
            return Result.Failure<List<PollVote>>(PollErrors.SingleChoiceOnly);

        if (Votes.Any(v => v.VoterId == voterId))
            return Result.Failure<List<PollVote>>(PollErrors.AlreadyVoted);

        var distinctOptionIds = optionIds.Distinct().ToList();
        var selectedOptions = Options.Where(o => distinctOptionIds.Contains(o.Id)).ToList();
        if (selectedOptions.Count != distinctOptionIds.Count)
            return Result.Failure<List<PollVote>>(PollErrors.OptionNotFound);

        var newVotes = new List<PollVote>();
        foreach (var option in selectedOptions)
        {
            option.IncrementVote();
            var vote = PollVote.Create(Id, option.Id, voterId);
            Votes.Add(vote);
            newVotes.Add(vote);
        }

        return Result.Success(newVotes);
    }

    public Result Close(Guid actorId, Guid meetingOwnerId)
    {
        if (actorId != CreatorId && actorId != meetingOwnerId)
            return Result.Failure(PollErrors.Unauthorized);

        if (!IsActive)
            return Result.Failure(PollErrors.Closed);

        IsActive = false;
        ClosedAt = DateTime.UtcNow;
        return Result.Success();
    }
}
