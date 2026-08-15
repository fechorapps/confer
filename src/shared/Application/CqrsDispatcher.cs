using Microsoft.Extensions.DependencyInjection;
using Confer.Shared.Application.Interfaces;

namespace Confer.Shared.Application;

public sealed class CqrsDispatcher(IServiceProvider serviceProvider) : ICqrsDispatcher
{
    public async Task<TResponse> SendAsync<TResponse>(ICommand<TResponse> command, CancellationToken cancellationToken = default)
    {
        var handlerType = typeof(ICommandHandler<,>).MakeGenericType(command.GetType(), typeof(TResponse));
        var handler = serviceProvider.GetRequiredService(handlerType);

        var method = handlerType.GetMethod(nameof(ICommandHandler<ICommand<TResponse>, TResponse>.HandleAsync));
        if (method is null)
        {
            throw new InvalidOperationException($"HandleAsync method not found on handler {handlerType.Name}");
        }

        var result = method.Invoke(handler, [command, cancellationToken]);
        if (result is Task<TResponse> task)
        {
            return await task;
        }

        throw new InvalidOperationException($"Handler for {command.GetType().Name} did not return a valid Task<{typeof(TResponse).Name}>");
    }

    public async Task<TResponse> QueryAsync<TResponse>(IQuery<TResponse> query, CancellationToken cancellationToken = default)
    {
        var handlerType = typeof(IQueryHandler<,>).MakeGenericType(query.GetType(), typeof(TResponse));
        var handler = serviceProvider.GetRequiredService(handlerType);

        var method = handlerType.GetMethod(nameof(IQueryHandler<IQuery<TResponse>, TResponse>.HandleAsync));
        if (method is null)
        {
            throw new InvalidOperationException($"HandleAsync method not found on handler {handlerType.Name}");
        }

        var result = method.Invoke(handler, [query, cancellationToken]);
        if (result is Task<TResponse> task)
        {
            return await task;
        }

        throw new InvalidOperationException($"Handler for {query.GetType().Name} did not return a valid Task<{typeof(TResponse).Name}>");
    }
}
