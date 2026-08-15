using System.Text;
using Confer.Application.Interfaces;
using Confer.Application.Meetings.Telephony;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Observability;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Telephony;

public class TwilioSipTelephonyBridge : ITelephonyBridgeService
{
    private readonly IConferDbContext _dbContext;
    private readonly ILogger<TwilioSipTelephonyBridge> _logger;
    private readonly string _primaryPhoneNumber;
    private readonly string _sipDomain;

    private static readonly List<string> DefaultGlobalNumbers = new()
    {
        "+1 (888) 555-CONFER (US Toll Free)",
        "+1 (415) 555-0199 (San Francisco, CA)",
        "+44 20 7946 0991 (London, UK)",
        "+49 30 1663 9921 (Berlin, Germany)",
        "+52 55 4169 8830 (Mexico City, Mexico)",
        "+81 3 4578 9912 (Tokyo, Japan)"
    };

    public TwilioSipTelephonyBridge(
        IConferDbContext dbContext,
        ILogger<TwilioSipTelephonyBridge> logger,
        IConfiguration configuration)
    {
        _dbContext = dbContext;
        _logger = logger;
        _primaryPhoneNumber = configuration["Telephony:PrimaryPhoneNumber"] ?? "+1 (888) 555-2663";
        _sipDomain = configuration["Telephony:SipDomain"] ?? "sip.confer.local";
    }

    public async Task<Result<PstnDialInInfoDto>> GetTelephonyInfoAsync(Guid meetingId, CancellationToken ct = default)
    {
        var meeting = await _dbContext.Meetings.FirstOrDefaultAsync(m => m.Id == meetingId, ct);
        if (meeting == null)
        {
            return Result.Failure<PstnDialInInfoDto>(MeetingErrors.NotFound);
        }

        var directSipUri = $"sip:{meeting.JoinCode}@{_sipDomain}";
        return Result.Success(new PstnDialInInfoDto(
            meeting.Id,
            meeting.Title,
            _primaryPhoneNumber,
            meeting.JoinCode,
            directSipUri,
            DefaultGlobalNumbers.AsReadOnly()
        ));
    }

    public async Task<Result<SipInboundCallResponseDto>> ProcessInboundSipCallAsync(HandleSipInboundCallCommand command, CancellationToken ct = default)
    {
        var digits = command.DigitsEntered?.Trim() ?? string.Empty;

        // 1. If no digits entered yet, return prompt asking for meeting PIN
        if (string.IsNullOrWhiteSpace(digits))
        {
            var promptTwiml =
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n" +
                "<Response>\n" +
                "    <Gather numDigits=\"6\" action=\"/api/telephony/sip-inbound\" method=\"POST\" timeout=\"10\">\n" +
                "        <Say voice=\"Polly.Amy-Neural\" language=\"en-US\">Welcome to Confer. Please enter your 6-digit meeting pin, followed by the pound key.</Say>\n" +
                "    </Gather>\n" +
                "    <Say voice=\"Polly.Amy-Neural\">We did not receive any input. Goodbye.</Say>\n" +
                "    <Hangup/>\n" +
                "</Response>";

            return Result.Success(new SipInboundCallResponseDto(
                false,
                promptTwiml,
                null,
                null
            ));
        }

        // 2. Validate Meeting Join Code
        var meeting = await _dbContext.Meetings.FirstOrDefaultAsync(m => m.JoinCode == digits, ct);
        if (meeting == null || meeting.EndedAt != null)
        {
            var notFoundTwiml =
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n" +
                "<Response>\n" +
                "    <Say voice=\"Polly.Amy-Neural\" language=\"en-US\">The meeting pin you entered is invalid or the meeting has ended. Please check the code and try again.</Say>\n" +
                "    <Gather numDigits=\"6\" action=\"/api/telephony/sip-inbound\" method=\"POST\" timeout=\"10\"/>\n" +
                "    <Hangup/>\n" +
                "</Response>";

            return Result.Success(new SipInboundCallResponseDto(
                false,
                notFoundTwiml,
                null,
                null
            ));
        }

        // 3. Connect to Conference Audio Room
        _logger.LogInformation("Bridging PSTN caller {Caller} into meeting {MeetingId} ({Title})",
            command.CallerNumber, meeting.Id, meeting.Title);

        var roomName = $"confer_room_{meeting.Id:N}";
        var connectTwiml =
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n" +
            "<Response>\n" +
            $"    <Say voice=\"Polly.Amy-Neural\" language=\"en-US\">Connecting you to {meeting.Title}. You are now in the conference.</Say>\n" +
            "    <Dial>\n" +
            $"        <Conference waitUrl=\"https://twimlets.com/holdmusic?Bucket=com.twilio.music.classical\" beep=\"true\" startConferenceOnEnter=\"true\" endConferenceOnExit=\"false\">{roomName}</Conference>\n" +
            "    </Dial>\n" +
            "</Response>";

        return Result.Success(new SipInboundCallResponseDto(
            true,
            connectTwiml,
            meeting.Title,
            meeting.Id
        ));
    }
}
