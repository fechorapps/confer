using Confer.Infrastructure.Signal;
using Confer.Shared.Api;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;

namespace Confer.Api.Endpoints.Signal;

public sealed class SignalingEndpoint : IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        app.Map("/v1/signal", async (HttpContext context, WebSocketSignalingHandler handler) =>
        {
            if (context.WebSockets.IsWebSocketRequest)
            {
                var token = context.Request.Query["token"].ToString();
                var subprotocol = context.WebSockets.WebSocketRequestedProtocols.Contains("confer.v1")
                    ? "confer.v1"
                    : null;
                using var webSocket = await context.WebSockets.AcceptWebSocketAsync(subprotocol);
                await handler.HandleWebSocketAsync(context, webSocket, token);
            }
            else
            {
                context.Response.StatusCode = StatusCodes.Status400BadRequest;
                await context.Response.WriteAsync("WebSocket connection expected.");
            }
        });
    }
}
