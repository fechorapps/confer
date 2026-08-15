using Confer.Shared.Results;

namespace Confer.Domain.Identity;

public sealed class User
{
    public Guid Id { get; private set; }
    public string Email { get; private set; } = string.Empty;
    public string DisplayName { get; private set; } = string.Empty;
    public string? AvatarUrl { get; private set; }
    public string? IdpSubject { get; private set; }
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;

    private User() { }

    public static Result<User> Create(
        string email,
        string displayName,
        string? avatarUrl = null,
        string? idpSubject = null)
    {
        if (string.IsNullOrWhiteSpace(email))
            return Result.Failure<User>(Error.Validation("User.EmailEmpty", "Email cannot be empty."));

        if (string.IsNullOrWhiteSpace(displayName))
            return Result.Failure<User>(Error.Validation("User.DisplayNameEmpty", "DisplayName cannot be empty."));

        return Result.Success(new User
        {
            Id = Guid.NewGuid(),
            Email = email.Trim().ToLowerInvariant(),
            DisplayName = displayName.Trim(),
            AvatarUrl = avatarUrl,
            IdpSubject = idpSubject,
            CreatedAt = DateTime.UtcNow
        });
    }

    public static User CreateWithId(
        Guid id,
        string email,
        string displayName,
        string? avatarUrl = null,
        string? idpSubject = null)
    {
        return new User
        {
            Id = id,
            Email = email.Trim().ToLowerInvariant(),
            DisplayName = displayName.Trim(),
            AvatarUrl = avatarUrl,
            IdpSubject = idpSubject,
            CreatedAt = DateTime.UtcNow
        };
    }
}
