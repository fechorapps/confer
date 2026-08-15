using Confer.Application.DTOs;
using Confer.Domain.Enums;

namespace Confer.Infrastructure.Media;

public sealed class SimulcastLayerSelector
{
    private readonly Dictionary<Guid, (SimulcastLayer CurrentLayer, DateTime LastUpgradeAttempt)> _layerStates = new();
    private static readonly TimeSpan UpgradeHysteresis = TimeSpan.FromSeconds(2.0); // SDD §6.2: 2s sustainable window

    public SimulcastLayer SelectLayer(Guid publisherId, TileSpec tile, int estimatedDownlinkKbps = 2000)
    {
        if (!tile.Visible)
            return SimulcastLayer.Q;

        // Desired layer based on rendered pixel size
        // f: > 640x360, h: > 320x180, q: <= 320x180
        var pixelArea = tile.Width * tile.Height;
        var desiredLayer = pixelArea switch
        {
            > 640 * 360 when estimatedDownlinkKbps >= 1000 => SimulcastLayer.F,
            > 320 * 180 when estimatedDownlinkKbps >= 400 => SimulcastLayer.H,
            _ => SimulcastLayer.Q
        };

        if (!_layerStates.TryGetValue(publisherId, out var state))
        {
            _layerStates[publisherId] = (desiredLayer, DateTime.UtcNow);
            return desiredLayer;
        }

        // Downgrade is immediate (SDD §6.2)
        if (desiredLayer > state.CurrentLayer)
        {
            _layerStates[publisherId] = (desiredLayer, DateTime.UtcNow);
            return desiredLayer;
        }

        // Upgrade requires 2-second hysteresis
        if (desiredLayer < state.CurrentLayer)
        {
            if (DateTime.UtcNow - state.LastUpgradeAttempt >= UpgradeHysteresis)
            {
                _layerStates[publisherId] = (desiredLayer, DateTime.UtcNow);
                return desiredLayer;
            }

            return state.CurrentLayer;
        }

        _layerStates[publisherId] = (state.CurrentLayer, DateTime.UtcNow);
        return state.CurrentLayer;
    }
}
