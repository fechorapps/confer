using Confer.Application.Interfaces;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Persistence;

public class LocalDiskRecordingStorage : IRecordingStorageService
{
    private readonly string _basePath;
    private readonly ILogger<LocalDiskRecordingStorage> _logger;

    public LocalDiskRecordingStorage(IConfiguration configuration, ILogger<LocalDiskRecordingStorage> logger)
    {
        _logger = logger;
        var configuredPath = configuration["Recording:StoragePath"];
        _basePath = !string.IsNullOrWhiteSpace(configuredPath)
            ? configuredPath
            : Path.Combine(Directory.GetCurrentDirectory(), "recordings");

        if (!Directory.Exists(_basePath))
        {
            Directory.CreateDirectory(_basePath);
        }
    }

    public string GetRecordingPath(Guid meetingId, Guid recordingId, string extension = "webm")
    {
        var cleanExt = extension.TrimStart('.');
        var meetingDir = Path.Combine(_basePath, meetingId.ToString());
        return Path.Combine(meetingDir, $"{recordingId}.{cleanExt}");
    }

    public async Task<string> SaveRecordingAsync(
        Guid meetingId,
        Guid recordingId,
        Stream stream,
        string extension = "webm",
        CancellationToken cancellationToken = default)
    {
        var filePath = GetRecordingPath(meetingId, recordingId, extension);
        var dir = Path.GetDirectoryName(filePath);
        if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
        {
            Directory.CreateDirectory(dir);
        }

        await using var fileStream = new FileStream(filePath, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
        if (stream.CanSeek)
        {
            stream.Position = 0;
        }
        await stream.CopyToAsync(fileStream, cancellationToken);
        _logger.LogInformation("Saved recording stream for meeting {MeetingId} at {FilePath}", meetingId, filePath);
        return filePath;
    }

    public async Task<string> SaveRecordingAsync(
        Guid meetingId,
        Guid recordingId,
        byte[] data,
        string extension = "webm",
        CancellationToken cancellationToken = default)
    {
        var filePath = GetRecordingPath(meetingId, recordingId, extension);
        var dir = Path.GetDirectoryName(filePath);
        if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
        {
            Directory.CreateDirectory(dir);
        }

        await File.WriteAllBytesAsync(filePath, data, cancellationToken);
        _logger.LogInformation("Saved recording bytes ({Length} bytes) for meeting {MeetingId} at {FilePath}", data.Length, meetingId, filePath);
        return filePath;
    }

    public Task<Stream?> GetRecordingStreamAsync(string storagePath, CancellationToken cancellationToken = default)
    {
        if (!File.Exists(storagePath))
        {
            return Task.FromResult<Stream?>(null);
        }

        Stream stream = new FileStream(storagePath, FileMode.Open, FileAccess.Read, FileShare.Read);
        return Task.FromResult<Stream?>(stream);
    }

    public Task<bool> DeleteRecordingAsync(string storagePath, CancellationToken cancellationToken = default)
    {
        if (File.Exists(storagePath))
        {
            try
            {
                File.Delete(storagePath);
                return Task.FromResult(true);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Failed to delete recording at {StoragePath}", storagePath);
                return Task.FromResult(false);
            }
        }
        return Task.FromResult(false);
    }

    public Task<long> GetRecordingSizeAsync(string storagePath, CancellationToken cancellationToken = default)
    {
        if (File.Exists(storagePath))
        {
            var info = new FileInfo(storagePath);
            return Task.FromResult(info.Length);
        }
        return Task.FromResult(0L);
    }

    public Task<bool> ExistsAsync(string storagePath, CancellationToken cancellationToken = default)
    {
        return Task.FromResult(File.Exists(storagePath));
    }
}
