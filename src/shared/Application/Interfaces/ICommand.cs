namespace Confer.Shared.Application.Interfaces;

public interface ICommand<out TResponse>;

public interface ICommand : ICommand<Results.Result>;
