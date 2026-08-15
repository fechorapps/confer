namespace Confer.Application.Interfaces;

public interface IRecordingStorageService
{
    Task<string> SaveRecordingAsync(Guid meetingId, Guid recordingId, Stream stream, string extension = "webm", CancellationToken cancellationToken = default);
    Task<string> SaveRecordingAsync(Guid meetingId, Guid recordingId, byte[] data, string extension = "webm", CancellationToken cancellationToken = default);
    Task<Stream?> GetRecordingStreamAsync(string storagePath, CancellationToken cancellationToken = default);
    Task<bool> DeleteRecordingAsync(string storagePath, CancellationToken cancellationToken = default);
    Task<long> GetRecordingSizeAsync(string storagePath, CancellationToken cancellationToken = default);
    Task<bool> ExistsAsync(string storagePath, CancellationToken cancellationToken = default);
    string GetRecordingPath(Guid meetingId, Guid recordingId, string extension = "webm");
}
