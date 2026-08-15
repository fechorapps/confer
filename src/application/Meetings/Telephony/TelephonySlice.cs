using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;

namespace Confer.Application.Meetings.Telephony;

public sealed record PstnDialInInfoDto(
    Guid MeetingId,
    string Title,
    string PhoneNumber,
    string ConferencePin,
    string DirectSipUri,
    IReadOnlyList<string> GlobalDialInNumbers
);

public sealed record GetMeetingTelephonyInfoQuery(
    Guid MeetingId
) : IQuery<Result<PstnDialInInfoDto>>;

public sealed class GetMeetingTelephonyInfoQueryValidator : AbstractValidator<GetMeetingTelephonyInfoQuery>
{
    public GetMeetingTelephonyInfoQueryValidator()
    {
        RuleFor(x => x.MeetingId).NotEmpty();
    }
}

public sealed record HandleSipInboundCallCommand(
    string CallerNumber,
    string? DigitsEntered,
    string? CallSid,
    string? FromCity,
    string? FromCountry
) : ICommand<Result<SipInboundCallResponseDto>>;

public sealed record SipInboundCallResponseDto(
    bool IsConnected,
    string ResponseTwiMl,
    string? MeetingTitle,
    Guid? MeetingId
);

public interface ITelephonyBridgeService
{
    Task<Result<PstnDialInInfoDto>> GetTelephonyInfoAsync(Guid meetingId, CancellationToken ct = default);
    Task<Result<SipInboundCallResponseDto>> ProcessInboundSipCallAsync(HandleSipInboundCallCommand command, CancellationToken ct = default);
}

public sealed class GetMeetingTelephonyInfoQueryHandler : IQueryHandler<GetMeetingTelephonyInfoQuery, Result<PstnDialInInfoDto>>
{
    private readonly ITelephonyBridgeService _telephonyService;

    public GetMeetingTelephonyInfoQueryHandler(ITelephonyBridgeService telephonyService)
    {
        _telephonyService = telephonyService;
    }

    public Task<Result<PstnDialInInfoDto>> HandleAsync(GetMeetingTelephonyInfoQuery request, CancellationToken cancellationToken = default)
    {
        return _telephonyService.GetTelephonyInfoAsync(request.MeetingId, cancellationToken);
    }
}

public sealed class HandleSipInboundCallCommandHandler : ICommandHandler<HandleSipInboundCallCommand, Result<SipInboundCallResponseDto>>
{
    private readonly ITelephonyBridgeService _telephonyService;

    public HandleSipInboundCallCommandHandler(ITelephonyBridgeService telephonyService)
    {
        _telephonyService = telephonyService;
    }

    public Task<Result<SipInboundCallResponseDto>> HandleAsync(HandleSipInboundCallCommand request, CancellationToken cancellationToken = default)
    {
        return _telephonyService.ProcessInboundSipCallAsync(request, cancellationToken);
    }
}
