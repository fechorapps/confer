using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Confer.Infrastructure.Persistence.Migrations
{
    /// <inheritdoc />
    public partial class InitialCreate : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "caption_chunks",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    SessionId = table.Column<Guid>(type: "TEXT", nullable: false),
                    ParticipantId = table.Column<Guid>(type: "TEXT", nullable: false),
                    SpeakerName = table.Column<string>(type: "TEXT", maxLength: 150, nullable: false),
                    Text = table.Column<string>(type: "TEXT", maxLength: 2000, nullable: false),
                    Language = table.Column<string>(type: "TEXT", maxLength: 16, nullable: false),
                    SpokenAt = table.Column<DateTime>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_caption_chunks", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "meetings",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    JoinCode = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false),
                    Title = table.Column<string>(type: "TEXT", maxLength: 200, nullable: false),
                    OwnerId = table.Column<Guid>(type: "TEXT", nullable: false),
                    ScheduledStart = table.Column<DateTime>(type: "TEXT", nullable: true),
                    MaxParticipants = table.Column<int>(type: "INTEGER", nullable: false),
                    IsLocked = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsWaitingRoomEnabled = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    IsWatermarkEnabled = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    Policy_AllowScreenShare = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: true),
                    Policy_AllowChat = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: true),
                    Policy_AllowUnmuteSelf = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: true),
                    Policy_MuteOnEntry = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: false),
                    Policy_AllowRename = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: true),
                    RecordingMode = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false),
                    IsRecording = table.Column<bool>(type: "INTEGER", nullable: false),
                    RecordingStartedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    LiveStreamConfig_IsStreamingEnabled = table.Column<bool>(type: "INTEGER", nullable: false),
                    LiveStreamConfig_RtmpUrl = table.Column<string>(type: "TEXT", maxLength: 500, nullable: false),
                    LiveStreamConfig_StreamKey = table.Column<string>(type: "TEXT", maxLength: 255, nullable: false),
                    LiveStreamConfig_Status = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false),
                    LiveStreamConfig_StartedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    EndedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_meetings", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "users",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    Email = table.Column<string>(type: "TEXT", maxLength: 255, nullable: false),
                    DisplayName = table.Column<string>(type: "TEXT", maxLength: 150, nullable: false),
                    AvatarUrl = table.Column<string>(type: "TEXT", nullable: true),
                    IdpSubject = table.Column<string>(type: "TEXT", maxLength: 255, nullable: true),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_users", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "webhook_subscriptions",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    UserId = table.Column<Guid>(type: "TEXT", nullable: false),
                    TargetUrl = table.Column<string>(type: "TEXT", maxLength: 1000, nullable: false),
                    Secret = table.Column<string>(type: "TEXT", maxLength: 255, nullable: false),
                    SubscribedEvents = table.Column<string>(type: "TEXT", nullable: false),
                    IsActive = table.Column<bool>(type: "INTEGER", nullable: false, defaultValue: true),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_webhook_subscriptions", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "breakout_rooms",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    MeetingId = table.Column<Guid>(type: "TEXT", nullable: false),
                    SessionId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Name = table.Column<string>(type: "TEXT", maxLength: 150, nullable: false),
                    Index = table.Column<int>(type: "INTEGER", nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    EndsAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    MaxDurationMinutes = table.Column<int>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_breakout_rooms", x => x.Id);
                    table.ForeignKey(
                        name: "FK_breakout_rooms_meetings_MeetingId",
                        column: x => x.MeetingId,
                        principalTable: "meetings",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "meeting_recordings",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    MeetingId = table.Column<Guid>(type: "TEXT", nullable: false),
                    InitiatedBy = table.Column<Guid>(type: "TEXT", nullable: false),
                    StartedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    StoppedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    DurationSeconds = table.Column<long>(type: "INTEGER", nullable: false),
                    FileSizeBytes = table.Column<long>(type: "INTEGER", nullable: false),
                    StoragePath = table.Column<string>(type: "TEXT", maxLength: 500, nullable: false),
                    StorageProvider = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    Status = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_meeting_recordings", x => x.Id);
                    table.ForeignKey(
                        name: "FK_meeting_recordings_meetings_MeetingId",
                        column: x => x.MeetingId,
                        principalTable: "meetings",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "meeting_summaries",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    MeetingId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Overview = table.Column<string>(type: "TEXT", maxLength: 4000, nullable: false),
                    KeyDecisions = table.Column<string>(type: "TEXT", nullable: false),
                    ActionItems = table.Column<string>(type: "TEXT", nullable: false),
                    GeneratedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    DurationMinutes = table.Column<int>(type: "INTEGER", nullable: false),
                    ParticipantCount = table.Column<int>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_meeting_summaries", x => x.Id);
                    table.ForeignKey(
                        name: "FK_meeting_summaries_meetings_MeetingId",
                        column: x => x.MeetingId,
                        principalTable: "meetings",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "polls",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    MeetingId = table.Column<Guid>(type: "TEXT", nullable: false),
                    CreatorId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Question = table.Column<string>(type: "TEXT", maxLength: 500, nullable: false),
                    IsAnonymous = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsMultiChoice = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsActive = table.Column<bool>(type: "INTEGER", nullable: false),
                    CreatedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    ClosedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_polls", x => x.Id);
                    table.ForeignKey(
                        name: "FK_polls_meetings_MeetingId",
                        column: x => x.MeetingId,
                        principalTable: "meetings",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "sessions",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    MeetingId = table.Column<Guid>(type: "TEXT", nullable: false),
                    SfuNodeId = table.Column<string>(type: "TEXT", maxLength: 100, nullable: false),
                    StartedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    EndedAt = table.Column<DateTime>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_sessions", x => x.Id);
                    table.ForeignKey(
                        name: "FK_sessions_meetings_MeetingId",
                        column: x => x.MeetingId,
                        principalTable: "meetings",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "poll_options",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    PollId = table.Column<Guid>(type: "TEXT", nullable: false),
                    Text = table.Column<string>(type: "TEXT", maxLength: 200, nullable: false),
                    Index = table.Column<int>(type: "INTEGER", nullable: false),
                    VoteCount = table.Column<int>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_poll_options", x => x.Id);
                    table.ForeignKey(
                        name: "FK_poll_options_polls_PollId",
                        column: x => x.PollId,
                        principalTable: "polls",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "poll_votes",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    PollId = table.Column<Guid>(type: "TEXT", nullable: false),
                    OptionId = table.Column<Guid>(type: "TEXT", nullable: false),
                    VoterId = table.Column<Guid>(type: "TEXT", nullable: false),
                    VotedAt = table.Column<DateTime>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_poll_votes", x => x.Id);
                    table.ForeignKey(
                        name: "FK_poll_votes_polls_PollId",
                        column: x => x.PollId,
                        principalTable: "polls",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "chat_messages",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    SessionId = table.Column<Guid>(type: "TEXT", nullable: false),
                    UserId = table.Column<Guid>(type: "TEXT", nullable: false),
                    UserName = table.Column<string>(type: "TEXT", maxLength: 150, nullable: false),
                    Body = table.Column<string>(type: "TEXT", maxLength: 2000, nullable: false),
                    SentAt = table.Column<DateTime>(type: "TEXT", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_chat_messages", x => x.Id);
                    table.ForeignKey(
                        name: "FK_chat_messages_sessions_SessionId",
                        column: x => x.SessionId,
                        principalTable: "sessions",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "participations",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    SessionId = table.Column<Guid>(type: "TEXT", nullable: false),
                    UserId = table.Column<Guid>(type: "TEXT", nullable: false),
                    DisplayName = table.Column<string>(type: "TEXT", maxLength: 150, nullable: false),
                    Role = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false),
                    Status = table.Column<string>(type: "TEXT", maxLength: 32, nullable: false),
                    JoinedAt = table.Column<DateTime>(type: "TEXT", nullable: false),
                    AdmittedAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    LeftAt = table.Column<DateTime>(type: "TEXT", nullable: true),
                    LeaveReason = table.Column<string>(type: "TEXT", maxLength: 32, nullable: true),
                    ClientInfo = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_participations", x => x.Id);
                    table.ForeignKey(
                        name: "FK_participations_sessions_SessionId",
                        column: x => x.SessionId,
                        principalTable: "sessions",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_breakout_rooms_MeetingId",
                table: "breakout_rooms",
                column: "MeetingId");

            migrationBuilder.CreateIndex(
                name: "IX_caption_chunks_SessionId_SpokenAt",
                table: "caption_chunks",
                columns: new[] { "SessionId", "SpokenAt" });

            migrationBuilder.CreateIndex(
                name: "IX_chat_messages_SessionId_SentAt",
                table: "chat_messages",
                columns: new[] { "SessionId", "SentAt" });

            migrationBuilder.CreateIndex(
                name: "IX_meeting_recordings_MeetingId",
                table: "meeting_recordings",
                column: "MeetingId");

            migrationBuilder.CreateIndex(
                name: "IX_meeting_summaries_MeetingId",
                table: "meeting_summaries",
                column: "MeetingId",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_meetings_JoinCode",
                table: "meetings",
                column: "JoinCode",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_participations_SessionId",
                table: "participations",
                column: "SessionId");

            migrationBuilder.CreateIndex(
                name: "IX_participations_UserId",
                table: "participations",
                column: "UserId");

            migrationBuilder.CreateIndex(
                name: "IX_poll_options_PollId",
                table: "poll_options",
                column: "PollId");

            migrationBuilder.CreateIndex(
                name: "IX_poll_votes_PollId_VoterId",
                table: "poll_votes",
                columns: new[] { "PollId", "VoterId" });

            migrationBuilder.CreateIndex(
                name: "IX_polls_MeetingId",
                table: "polls",
                column: "MeetingId");

            migrationBuilder.CreateIndex(
                name: "IX_sessions_MeetingId",
                table: "sessions",
                column: "MeetingId");

            migrationBuilder.CreateIndex(
                name: "IX_users_Email",
                table: "users",
                column: "Email",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_users_IdpSubject",
                table: "users",
                column: "IdpSubject",
                unique: true,
                filter: "\"IdpSubject\" IS NOT NULL");

            migrationBuilder.CreateIndex(
                name: "IX_webhook_subscriptions_UserId",
                table: "webhook_subscriptions",
                column: "UserId");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "breakout_rooms");

            migrationBuilder.DropTable(
                name: "caption_chunks");

            migrationBuilder.DropTable(
                name: "chat_messages");

            migrationBuilder.DropTable(
                name: "meeting_recordings");

            migrationBuilder.DropTable(
                name: "meeting_summaries");

            migrationBuilder.DropTable(
                name: "participations");

            migrationBuilder.DropTable(
                name: "poll_options");

            migrationBuilder.DropTable(
                name: "poll_votes");

            migrationBuilder.DropTable(
                name: "users");

            migrationBuilder.DropTable(
                name: "webhook_subscriptions");

            migrationBuilder.DropTable(
                name: "sessions");

            migrationBuilder.DropTable(
                name: "polls");

            migrationBuilder.DropTable(
                name: "meetings");
        }
    }
}
