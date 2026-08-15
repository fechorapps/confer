using Confer.Application.Meetings.Telephony;
using Confer.Shared.Api;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Routing;

namespace Confer.Api.Endpoints.Meetings;

public sealed class TelephonyEndpoint : BaseModule, IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        // Meeting Telephony Info
        app.MapGet("/api/meetings/{id:guid}/telephony", GetMeetingTelephony)
            .WithTags("Telephony")
            .WithName("GetMeetingTelephony");

        // Twilio / SIP Trunking Inbound Webhook
        app.MapPost("/api/telephony/sip-inbound", HandleSipInbound)
            .WithTags("Telephony")
            .WithName("HandleSipInbound");
    }

    private static async Task<IResult> GetMeetingTelephony(
        [FromRoute] Guid id,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var result = await dispatcher.QueryAsync(new GetMeetingTelephonyInfoQuery(id), ct);
        return result.IsSuccess
            ? TypedResults.Ok(result.Value)
            : TypedResults.NotFound(new { result.Error.Code, result.Error.Description });
    }

    private static async Task<IResult> HandleSipInbound(
        HttpRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        string? digits = null;
        string? fromNumber = null;
        string? callSid = null;
        string? fromCity = null;
        string? fromCountry = null;

        if (request.HasFormContentType)
        {
            var form = await request.ReadFormAsync(ct);
            digits = form["Digits"].ToString();
            fromNumber = form["From"].ToString();
            callSid = form["CallSid"].ToString();
            fromCity = form["FromCity"].ToString();
            fromCountry = form["FromCountry"].ToString();
        }

        var command = new HandleSipInboundCallCommand(
            fromNumber ?? "Anonymous",
            digits,
            callSid,
            fromCity,
            fromCountry
        );

        var result = await dispatcher.SendAsync(command, ct);
        if (result.IsFailure)
        {
            return TypedResults.Content(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Response><Say>An error occurred.</Say><Hangup/></Response>",
                "application/xml");
        }

        return TypedResults.Content(result.Value.ResponseTwiMl, "application/xml");
    }
}
