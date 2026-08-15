using Confer.Domain.Meetings;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class PollDomainTests
{
    [Fact]
    public void CreatePoll_WithValidParameters_ShouldSucceed()
    {
        var meetingId = Guid.NewGuid();
        var creatorId = Guid.NewGuid();
        var options = new List<string> { "Option A", "Option B", "Option C" };

        var result = Poll.Create(meetingId, creatorId, "What is your favorite language?", options);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meetingId);
        result.Value.CreatorId.Should().Be(creatorId);
        result.Value.Question.Should().Be("What is your favorite language?");
        result.Value.IsActive.Should().BeTrue();
        result.Value.Options.Should().HaveCount(3);
        result.Value.Votes.Should().BeEmpty();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void CreatePoll_WithEmptyQuestion_ShouldFail(string question)
    {
        var result = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), question, new List<string> { "A", "B" });
        result.IsFailure.Should().BeTrue();
    }

    [Fact]
    public void CreatePoll_WithLessThanTwoOptions_ShouldFail()
    {
        var result = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), "Question?", new List<string> { "Single option" });
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(PollErrors.InvalidOptions);
    }

    [Fact]
    public void Vote_SingleChoice_ShouldIncrementAndRecordVote()
    {
        var poll = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), "Pick one", new List<string> { "A", "B" }, isMultiChoice: false).Value;
        var firstOption = poll.Options.First();
        var voterId = Guid.NewGuid();

        var voteResult = poll.Vote(voterId, new List<Guid> { firstOption.Id });

        voteResult.IsSuccess.Should().BeTrue();
        voteResult.Value.Should().HaveCount(1);
        firstOption.VoteCount.Should().Be(1);
        poll.Votes.Should().HaveCount(1);
    }

    [Fact]
    public void Vote_SingleChoice_WhenVotingMultipleOptions_ShouldFail()
    {
        var poll = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), "Pick one", new List<string> { "A", "B" }, isMultiChoice: false).Value;
        var optionIds = poll.Options.Select(o => o.Id).ToList();
        var voterId = Guid.NewGuid();

        var voteResult = poll.Vote(voterId, optionIds);

        voteResult.IsFailure.Should().BeTrue();
        voteResult.Error.Should().Be(PollErrors.SingleChoiceOnly);
    }

    [Fact]
    public void Vote_MultiChoice_ShouldIncrementAllSelectedOptions()
    {
        var poll = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), "Pick many", new List<string> { "A", "B", "C" }, isMultiChoice: true).Value;
        var optionIds = poll.Options.Take(2).Select(o => o.Id).ToList();
        var voterId = Guid.NewGuid();

        var voteResult = poll.Vote(voterId, optionIds);

        voteResult.IsSuccess.Should().BeTrue();
        voteResult.Value.Should().HaveCount(2);
        poll.Options.ElementAt(0).VoteCount.Should().Be(1);
        poll.Options.ElementAt(1).VoteCount.Should().Be(1);
        poll.Options.ElementAt(2).VoteCount.Should().Be(0);
        poll.Votes.Should().HaveCount(2);
    }

    [Fact]
    public void Vote_WhenAlreadyVoted_ShouldFail()
    {
        var poll = Poll.Create(Guid.NewGuid(), Guid.NewGuid(), "Pick one", new List<string> { "A", "B" }).Value;
        var firstOption = poll.Options.First();
        var voterId = Guid.NewGuid();

        poll.Vote(voterId, new List<Guid> { firstOption.Id });
        var secondVote = poll.Vote(voterId, new List<Guid> { firstOption.Id });

        secondVote.IsFailure.Should().BeTrue();
        secondVote.Error.Should().Be(PollErrors.AlreadyVoted);
    }

    [Fact]
    public void Vote_WhenPollClosed_ShouldFail()
    {
        var creatorId = Guid.NewGuid();
        var ownerId = Guid.NewGuid();
        var poll = Poll.Create(Guid.NewGuid(), creatorId, "Pick one", new List<string> { "A", "B" }).Value;
        poll.Close(creatorId, ownerId);

        var voteResult = poll.Vote(Guid.NewGuid(), new List<Guid> { poll.Options.First().Id });

        voteResult.IsFailure.Should().BeTrue();
        voteResult.Error.Should().Be(PollErrors.Closed);
    }

    [Fact]
    public void Close_ByCreator_ShouldSucceed()
    {
        var creatorId = Guid.NewGuid();
        var ownerId = Guid.NewGuid();
        var poll = Poll.Create(Guid.NewGuid(), creatorId, "Pick one", new List<string> { "A", "B" }).Value;

        var result = poll.Close(creatorId, ownerId);

        result.IsSuccess.Should().BeTrue();
        poll.IsActive.Should().BeFalse();
        poll.ClosedAt.Should().NotBeNull();
    }

    [Fact]
    public void Close_ByMeetingOwner_ShouldSucceed()
    {
        var creatorId = Guid.NewGuid();
        var ownerId = Guid.NewGuid();
        var poll = Poll.Create(Guid.NewGuid(), creatorId, "Pick one", new List<string> { "A", "B" }).Value;

        var result = poll.Close(ownerId, ownerId);

        result.IsSuccess.Should().BeTrue();
        poll.IsActive.Should().BeFalse();
    }

    [Fact]
    public void Close_ByUnauthorizedUser_ShouldFail()
    {
        var creatorId = Guid.NewGuid();
        var ownerId = Guid.NewGuid();
        var unauthorizedId = Guid.NewGuid();
        var poll = Poll.Create(Guid.NewGuid(), creatorId, "Pick one", new List<string> { "A", "B" }).Value;

        var result = poll.Close(unauthorizedId, ownerId);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(PollErrors.Unauthorized);
    }
}
