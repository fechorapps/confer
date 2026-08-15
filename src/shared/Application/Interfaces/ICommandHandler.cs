namespace Confer.Shared.Application.Interfaces;

public interface ICommandHandler<in TCommand, TResponse>
    where TCommand : ICommand<TResponse>
{
    Task<TResponse> HandleAsync(TCommand command, CancellationToken cancellationToken = default);
}

public interface ICommandHandler<in TCommand> : ICommandHandler<TCommand, Results.Result>
    where TCommand : ICommand<Results.Result>;
