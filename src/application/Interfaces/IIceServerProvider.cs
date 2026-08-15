using Confer.Application.DTOs;

namespace Confer.Application.Interfaces;

public interface IIceServerProvider
{
    List<IceServerConfig> GetIceServers();
}
