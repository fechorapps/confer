using System.Reflection;
using Confer.Shared.Application;
using Confer.Shared.Application.Interfaces;
using FluentValidation;
using Microsoft.Extensions.DependencyInjection;

namespace Confer.Application;

public static class ServiceExtensions
{
    public static IServiceCollection AddApplication(this IServiceCollection services)
    {
        var assembly = Assembly.GetExecutingAssembly();

        services.AddScoped<ICqrsDispatcher, CqrsDispatcher>();

        // Register all ICommandHandler implementations
        var commandHandlerTypes = assembly.GetTypes()
            .Where(t => !t.IsAbstract && !t.IsInterface)
            .SelectMany(t => t.GetInterfaces(), (t, i) => new { Implementation = t, Interface = i })
            .Where(x => x.Interface.IsGenericType && x.Interface.GetGenericTypeDefinition() == typeof(ICommandHandler<,>));

        foreach (var handler in commandHandlerTypes)
        {
            services.AddScoped(handler.Interface, handler.Implementation);
        }

        // Register all IQueryHandler implementations
        var queryHandlerTypes = assembly.GetTypes()
            .Where(t => !t.IsAbstract && !t.IsInterface)
            .SelectMany(t => t.GetInterfaces(), (t, i) => new { Implementation = t, Interface = i })
            .Where(x => x.Interface.IsGenericType && x.Interface.GetGenericTypeDefinition() == typeof(IQueryHandler<,>));

        foreach (var handler in queryHandlerTypes)
        {
            services.AddScoped(handler.Interface, handler.Implementation);
        }

        // Register FluentValidation validators
        services.AddValidatorsFromAssembly(assembly);

        // Register AI Summary Service
        services.AddScoped<Confer.Application.Meetings.Summary.IAiSummaryService, Confer.Application.Meetings.Summary.SmartAiSummaryService>();

        return services;
    }
}
