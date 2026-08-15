using Confer.Application.DTOs;
using Confer.Domain.Enums;
using Confer.Infrastructure.Media;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class LayerAllocatorTests
{
    [Fact]
    public void SelectLayer_WhenTileIsHidden_ShouldReturnQuarterLayer()
    {
        var selector = new SimulcastLayerSelector();
        var publisherId = Guid.NewGuid();
        var tile = new TileSpec(publisherId, 1280, 720, Visible: false);

        var layer = selector.SelectLayer(publisherId, tile, 2000);

        layer.Should().Be(SimulcastLayer.Q);
    }

    [Fact]
    public void SelectLayer_WhenTileIsLargeAndBandwidthHigh_ShouldReturnFullLayer()
    {
        var selector = new SimulcastLayerSelector();
        var publisherId = Guid.NewGuid();
        var tile = new TileSpec(publisherId, 1280, 720, Visible: true);

        var layer = selector.SelectLayer(publisherId, tile, 2500);

        layer.Should().Be(SimulcastLayer.F);
    }

    [Fact]
    public void SelectLayer_WhenTileIsMedium_ShouldReturnHalfLayer()
    {
        var selector = new SimulcastLayerSelector();
        var publisherId = Guid.NewGuid();
        var tile = new TileSpec(publisherId, 480, 270, Visible: true);

        var layer = selector.SelectLayer(publisherId, tile, 800);

        layer.Should().Be(SimulcastLayer.H);
    }

    [Fact]
    public void SelectLayer_WhenDowngrading_ShouldBeImmediate()
    {
        var selector = new SimulcastLayerSelector();
        var publisherId = Guid.NewGuid();
        var largeTile = new TileSpec(publisherId, 1280, 720, Visible: true);
        var smallTile = new TileSpec(publisherId, 160, 90, Visible: true);

        var layer1 = selector.SelectLayer(publisherId, largeTile, 2000);
        layer1.Should().Be(SimulcastLayer.F);

        var layer2 = selector.SelectLayer(publisherId, smallTile, 2000);
        layer2.Should().Be(SimulcastLayer.Q);
    }
}
