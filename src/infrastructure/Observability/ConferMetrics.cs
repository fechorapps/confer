using System.Diagnostics.Metrics;

namespace Confer.Infrastructure.Observability;

/// <summary>
/// High-performance Prometheus and OpenTelemetry metrics for Confer WebRTC & Signaling Platform.
/// </summary>
public static class ConferMetrics
{
    public const string MeterName = "Confer.Platform";
    private static readonly Meter Meter = new(MeterName, "1.0.0");

    // Gauges / UpDownCounters
    public static readonly UpDownCounter<long> ActiveMeetings = Meter.CreateUpDownCounter<long>(
        "confer_active_meetings",
        description: "Number of active meeting rooms currently running");

    public static readonly UpDownCounter<long> ActiveParticipants = Meter.CreateUpDownCounter<long>(
        "confer_active_participants",
        description: "Number of participants currently connected across all rooms");

    public static readonly UpDownCounter<long> WebSocketConnections = Meter.CreateUpDownCounter<long>(
        "confer_websocket_connections_active",
        description: "Number of active WebSocket signaling connections");

    public static readonly UpDownCounter<long> SfuSessionsActive = Meter.CreateUpDownCounter<long>(
        "confer_sfu_sessions_active",
        description: "Number of active WebRTC SFU media sessions");

    // Counters
    public static readonly Counter<long> MessagesBroadcastTotal = Meter.CreateCounter<long>(
        "confer_messages_broadcast_total",
        description: "Total count of WebSocket signaling messages broadcasted");

    public static readonly Counter<long> PollsCreatedTotal = Meter.CreateCounter<long>(
        "confer_polls_created_total",
        description: "Total number of live polls created");

    public static readonly Counter<long> CaptionsStreamedTotal = Meter.CreateCounter<long>(
        "confer_captions_streamed_total",
        description: "Total count of real-time speech-to-text caption chunks streamed");

    public static readonly Counter<long> SummariesGeneratedTotal = Meter.CreateCounter<long>(
        "confer_summaries_generated_total",
        description: "Total count of AI meeting summaries generated");

    public static readonly Counter<long> AiCopilotQueriesTotal = Meter.CreateCounter<long>(
        "confer_ai_copilot_queries_total",
        description: "Total count of @confer-ai in-room companion queries handled");

    public static readonly Counter<long> WebhookEventsDispatchedTotal = Meter.CreateCounter<long>(
        "confer_webhook_events_dispatched_total",
        description: "Total count of webhook HTTP event deliveries dispatched");

    // Histograms
    public static readonly Histogram<double> SignalingLatencyMs = Meter.CreateHistogram<double>(
        "confer_signaling_latency_ms",
        unit: "ms",
        description: "WebSocket signaling dispatch latency in milliseconds");

    public static readonly Histogram<double> SfuPacketLossRatio = Meter.CreateHistogram<double>(
        "confer_sfu_packet_loss_ratio",
        description: "WebRTC SFU inbound/outbound packet loss ratio");
}
