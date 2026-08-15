using Microsoft.AspNetCore.Routing;

namespace Confer.Shared.Api;

public interface IEndpoint
{
    void MapEndpoint(IEndpointRouteBuilder app);
}
