using System.Text;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging.Abstractions;
using Xunit;

namespace Confer.Tests;

public class RecordingStorageServiceTests : IDisposable
{
    private readonly string _tempDir;
    private readonly LocalDiskRecordingStorage _storage;

    public RecordingStorageServiceTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "confer_test_recordings_" + Guid.NewGuid());
        Directory.CreateDirectory(_tempDir);

        var config = new ConfigurationBuilder()
            .AddInMemoryCollection(new Dictionary<string, string?>
            {
                ["Recording:StoragePath"] = _tempDir
            })
            .Build();

        _storage = new LocalDiskRecordingStorage(config, NullLogger<LocalDiskRecordingStorage>.Instance);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempDir))
        {
            try
            {
                Directory.Delete(_tempDir, true);
            }
            catch { }
        }
    }

    [Fact]
    public async Task SaveRecordingAsync_WithBytes_ShouldWriteToDiskAndReturnPath()
    {
        var meetingId = Guid.NewGuid();
        var recordingId = Guid.NewGuid();
        var data = Encoding.UTF8.GetBytes("WEBM_MOCK_VIDEO_STREAM_DATA");

        var path = await _storage.SaveRecordingAsync(meetingId, recordingId, data, "webm");

        path.Should().NotBeNullOrWhiteSpace();
        File.Exists(path).Should().BeTrue();
        (await _storage.ExistsAsync(path)).Should().BeTrue();

        var size = await _storage.GetRecordingSizeAsync(path);
        size.Should().Be(data.Length);
    }

    [Fact]
    public async Task SaveRecordingAsync_WithStream_ShouldWriteToDiskAndAllowReadback()
    {
        var meetingId = Guid.NewGuid();
        var recordingId = Guid.NewGuid();
        var sampleText = "VP8_MEDIA_CONTAINER_STREAM_PACKET";
        using var stream = new MemoryStream(Encoding.UTF8.GetBytes(sampleText));

        var path = await _storage.SaveRecordingAsync(meetingId, recordingId, stream, "webm");

        path.Should().NotBeNullOrWhiteSpace();
        File.Exists(path).Should().BeTrue();

        using var readStream = await _storage.GetRecordingStreamAsync(path);
        readStream.Should().NotBeNull();
        using var reader = new StreamReader(readStream!);
        var readContent = await reader.ReadToEndAsync();
        readContent.Should().Be(sampleText);
    }

    [Fact]
    public async Task DeleteRecordingAsync_ShouldRemoveFile()
    {
        var meetingId = Guid.NewGuid();
        var recordingId = Guid.NewGuid();
        var data = Encoding.UTF8.GetBytes("SAMPLE_RECORDING");

        var path = await _storage.SaveRecordingAsync(meetingId, recordingId, data, "webm");
        (await _storage.ExistsAsync(path)).Should().BeTrue();

        var deleted = await _storage.DeleteRecordingAsync(path);
        deleted.Should().BeTrue();
        (await _storage.ExistsAsync(path)).Should().BeFalse();
    }

    [Fact]
    public async Task GetRecordingStreamAsync_WhenNonExistent_ShouldReturnNull()
    {
        var result = await _storage.GetRecordingStreamAsync("/path/to/nonexistent/recording.webm");
        result.Should().BeNull();
    }
}
