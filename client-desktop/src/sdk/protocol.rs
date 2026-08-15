use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackIntent {
    pub track_id: String,
    pub kind: String,
    #[serde(default)]
    pub simulcast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSpec {
    pub participant_id: Uuid,
    pub width: i32,
    pub height: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantState {
    pub participant_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub role: String,
    #[serde(default)]
    pub is_audio_muted: bool,
    #[serde(default)]
    pub is_video_muted: bool,
    #[serde(default)]
    pub is_screen_sharing: bool,
    #[serde(default)]
    pub is_hand_raised: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerInfo {
    pub participant_id: Uuid,
    pub level_dbov: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub body: String,
    pub sent_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhiteboardColorDto {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl WhiteboardColorDto {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum WhiteboardShapeDto {
    Pen {
        points: Vec<[f32; 2]>,
    },
    Line {
        start: [f32; 2],
        end: [f32; 2],
    },
    Rectangle {
        start: [f32; 2],
        end: [f32; 2],
    },
    Circle {
        center: [f32; 2],
        radius: f32,
    },
    Text {
        pos: [f32; 2],
        text: String,
        font_size: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhiteboardStrokeDto {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub shape: WhiteboardShapeDto,
    pub color: WhiteboardColorDto,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollOptionDto {
    pub id: usize,
    pub text: String,
    #[serde(default)]
    pub vote_count: u32,
    #[serde(default)]
    pub voter_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollDto {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub creator_name: String,
    pub question: String,
    pub options: Vec<PollOptionDto>,
    #[serde(default)]
    pub multi_choice: bool,
    #[serde(default)]
    pub is_anonymous: bool,
    #[serde(default)]
    pub is_closed: bool,
    #[serde(default)]
    pub total_votes: u32,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingPolicyDto {
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub waiting_room_enabled: bool,
    #[serde(default = "default_true")]
    pub allow_screen_share: bool,
    #[serde(default = "default_true")]
    pub allow_chat: bool,
    #[serde(default = "default_true")]
    pub allow_unmute: bool,
    #[serde(default)]
    pub watermark_enabled: bool,
}

impl Default for MeetingPolicyDto {
    fn default() -> Self {
        Self {
            is_locked: false,
            waiting_room_enabled: false,
            allow_screen_share: true,
            allow_chat: true,
            allow_unmute: true,
            watermark_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitingParticipantDto {
    pub participant_id: Uuid,
    #[serde(default)]
    pub user_id: Uuid,
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub joined_lobby_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionChunkDto {
    pub participant_id: Uuid,
    pub speaker_name: String,
    pub text: String,
    #[serde(default)]
    pub is_final: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub timestamp_ms: u64,
}

fn default_language() -> String {
    "en-US".to_string()
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Publish {
        sdp: String,
        tracks: Vec<TrackIntent>,
    },
    Unpublish {
        track_id: String,
    },
    SubscribeAnswer {
        sdp: String,
    },
    UpdateViewport {
        tiles: Vec<TileSpec>,
    },
    SetMute {
        kind: String,
        muted: bool,
    },
    Chat {
        body: String,
        client_msg_id: Uuid,
    },
    Reaction {
        emoji: String,
    },
    HostAction {
        action: String,
        target_participant_id: Uuid,
    },
    CreatePoll {
        poll_id: Uuid,
        question: String,
        options: Vec<String>,
        multi_choice: bool,
        is_anonymous: bool,
    },
    VotePoll {
        poll_id: Uuid,
        selected_options: Vec<usize>,
    },
    ClosePoll {
        poll_id: Uuid,
    },
    WhiteboardStroke {
        stroke: WhiteboardStrokeDto,
    },
    WhiteboardClear,
    AdmitParticipant {
        participant_id: Uuid,
    },
    AdmitAll,
    RejectParticipant {
        participant_id: Uuid,
    },
    UpdatePolicy {
        policy: MeetingPolicyDto,
    },
    ToggleWaitingRoom {
        enabled: bool,
    },
    StartStream {
        rtmp_url: String,
        stream_key: String,
    },
    StopStream,
    #[serde(alias = "caption_chunk")]
    SendCaption {
        text: String,
        #[serde(default)]
        is_final: bool,
        #[serde(default = "default_language")]
        language: String,
        #[serde(default)]
        timestamp_ms: u64,
    },
    Ping {
        seq: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Joined {
        participant_id: Uuid,
        meeting_id: Uuid,
        room_title: String,
        role: String,
        roster: Vec<ParticipantState>,
    },
    PublishOk {
        sdp: String,
    },
    SubscribeOffer {
        sdp: String,
        mapping: Vec<serde_json::Value>,
    },
    ParticipantJoined {
        participant: ParticipantState,
    },
    ParticipantLeft {
        participant_id: Uuid,
        reason: String,
    },
    ParticipantMuteChanged {
        participant_id: Uuid,
        kind: String,
        muted: bool,
    },
    ActiveSpeakers {
        ranked: Vec<SpeakerInfo>,
    },
    Chat {
        id: Uuid,
        from_id: Uuid,
        from_name: String,
        body: String,
        sent_at: String,
    },
    Reaction {
        from_id: Uuid,
        from_name: String,
        emoji: String,
    },
    #[serde(alias = "caption_chunk")]
    CaptionBroadcast {
        participant_id: Uuid,
        speaker_name: String,
        text: String,
        #[serde(default)]
        is_final: bool,
        #[serde(default = "default_language")]
        language: String,
        #[serde(default)]
        timestamp_ms: u64,
    },
    PollCreated {

        poll: PollDto,
    },
    PollUpdated {
        poll: PollDto,
    },
    PollClosed {
        poll_id: Uuid,
    },
    WhiteboardStroke {
        stroke: WhiteboardStrokeDto,
    },
    WhiteboardClear,
    MeetingLocked {
        is_locked: bool,
    },
    WaitingRoomStatus {
        is_waiting: bool,
        #[serde(default)]
        message: Option<String>,
    },
    ParticipantWaiting {
        participant: WaitingParticipantDto,
    },
    ParticipantAdmitted {
        participant_id: Uuid,
    },
    ParticipantRejected {
        participant_id: Uuid,
    },
    MeetingPolicyChanged {
        policy: MeetingPolicyDto,
    },
    LiveStreamStatus {
        is_streaming: bool,
        #[serde(default)]
        rtmp_url: String,
        status: String,
        #[serde(default)]
        started_at: Option<String>,
    },
    MeetingEnded {
        reason: String,
    },
    Pong {
        seq: u64,
    },
    Error {
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_ping_serialization() {
        let msg = ClientMessage::Ping { seq: 42 };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Ping message");
        assert_eq!(json, r#"{"type":"ping","seq":42}"#);

        let parsed: ClientMessage = serde_json::from_str(&json).expect("Failed to deserialize Ping message");
        match parsed {
            ClientMessage::Ping { seq } => assert_eq!(seq, 42),
            _ => panic!("Expected Ping variant"),
        }
    }

    #[test]
    fn test_client_message_chat_serialization() {
        let client_id = Uuid::new_v4();
        let msg = ClientMessage::Chat {
            body: "Hello Confer!".to_string(),
            client_msg_id: client_id,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Chat message");
        assert!(json.contains(r#""type":"chat""#));
        assert!(json.contains("Hello Confer!"));
    }

    #[test]
    fn test_server_message_joined_deserialization() {
        let part_id = Uuid::new_v4();
        let meet_id = Uuid::new_v4();
        let raw_json = format!(
            r#"{{"type":"joined","participant_id":"{part_id}","meeting_id":"{meet_id}","room_title":"Engineering Standup","role":"host","roster":[]}}"#
        );

        let msg: ServerMessage = serde_json::from_str(&raw_json).expect("Failed to deserialize Joined");
        match msg {
            ServerMessage::Joined { participant_id, room_title, role, .. } => {
                assert_eq!(participant_id, part_id);
                assert_eq!(room_title, "Engineering Standup");
                assert_eq!(role, "host");
            }
            _ => panic!("Expected Joined variant"),
        }
    }

    #[test]
    fn test_server_message_pong_deserialization() {
        let raw_json = r#"{"type":"pong","seq":100}"#;
        let msg: ServerMessage = serde_json::from_str(raw_json).expect("Failed to deserialize Pong");
        match msg {
            ServerMessage::Pong { seq } => assert_eq!(seq, 100),
            _ => panic!("Expected Pong variant"),
        }
    }

    #[test]
    fn test_poll_messages_serialization_and_deserialization() {
        let poll_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let create_msg = ClientMessage::CreatePoll {
            poll_id,
            question: "What feature should we build next?".to_string(),
            options: vec!["Whiteboard".to_string(), "Polls".to_string(), "Breakout Rooms".to_string()],
            multi_choice: false,
            is_anonymous: true,
        };
        let json = serde_json::to_string(&create_msg).expect("Failed to serialize CreatePoll");
        assert!(json.contains(r#""type":"create_poll""#));
        assert!(json.contains("What feature should we build next?"));

        let vote_msg = ClientMessage::VotePoll {
            poll_id,
            selected_options: vec![0, 1],
        };
        let vote_json = serde_json::to_string(&vote_msg).expect("Failed to serialize VotePoll");
        assert!(vote_json.contains(r#""type":"vote_poll""#));

        let close_msg = ClientMessage::ClosePoll { poll_id };
        let close_json = serde_json::to_string(&close_msg).expect("Failed to serialize ClosePoll");
        assert!(close_json.contains(r#""type":"close_poll""#));

        let poll_dto = PollDto {
            id: poll_id,
            creator_id,
            creator_name: "Alice Host".to_string(),
            question: "Release v1.0 today?".to_string(),
            options: vec![
                PollOptionDto { id: 0, text: "Yes".to_string(), vote_count: 5, voter_ids: vec![creator_id] },
                PollOptionDto { id: 1, text: "No".to_string(), vote_count: 1, voter_ids: vec![] },
            ],
            multi_choice: false,
            is_anonymous: false,
            is_closed: false,
            total_votes: 6,
            created_at: "2026-08-14T20:00:00Z".to_string(),
        };

        let server_created = ServerMessage::PollCreated { poll: poll_dto.clone() };
        let srv_json = serde_json::to_string(&server_created).expect("Failed to serialize PollCreated");
        let parsed_srv: ServerMessage = serde_json::from_str(&srv_json).expect("Failed to deserialize PollCreated");
        match parsed_srv {
            ServerMessage::PollCreated { poll } => {
                assert_eq!(poll.id, poll_id);
                assert_eq!(poll.question, "Release v1.0 today?");
                assert_eq!(poll.options.len(), 2);
                assert_eq!(poll.options[0].vote_count, 5);
            }
            _ => panic!("Expected PollCreated variant"),
        }

        let server_closed = ServerMessage::PollClosed { poll_id };
        let closed_json = serde_json::to_string(&server_closed).expect("Failed to serialize PollClosed");
        let parsed_closed: ServerMessage = serde_json::from_str(&closed_json).expect("Failed to deserialize PollClosed");
        match parsed_closed {
            ServerMessage::PollClosed { poll_id: id } => assert_eq!(id, poll_id),
            _ => panic!("Expected PollClosed variant"),
        }
    }

    #[test]
    fn test_whiteboard_messages_serialization_and_deserialization() {
        let stroke_id = Uuid::new_v4();
        let part_id = Uuid::new_v4();

        let stroke_dto = WhiteboardStrokeDto {
            id: stroke_id,
            participant_id: part_id,
            shape: WhiteboardShapeDto::Rectangle {
                start: [10.0, 20.0],
                end: [100.0, 150.0],
            },
            color: WhiteboardColorDto::new(239, 68, 68, 255),
            stroke_width: 3.5,
        };

        let client_stroke = ClientMessage::WhiteboardStroke { stroke: stroke_dto.clone() };
        let json = serde_json::to_string(&client_stroke).expect("Failed to serialize WhiteboardStroke");
        assert!(json.contains(r#""type":"whiteboard_stroke""#));
        assert!(json.contains(r#""tool":"rectangle""#));

        let client_clear = ClientMessage::WhiteboardClear;
        let clear_json = serde_json::to_string(&client_clear).expect("Failed to serialize WhiteboardClear");
        assert_eq!(clear_json, r#"{"type":"whiteboard_clear"}"#);

        let server_stroke = ServerMessage::WhiteboardStroke { stroke: stroke_dto };
        let srv_json = serde_json::to_string(&server_stroke).expect("Failed to serialize Server WhiteboardStroke");
        let parsed_stroke: ServerMessage = serde_json::from_str(&srv_json).expect("Failed to deserialize Server WhiteboardStroke");
        match parsed_stroke {
            ServerMessage::WhiteboardStroke { stroke } => {
                assert_eq!(stroke.id, stroke_id);
                assert_eq!(stroke.color.r, 239);
                assert_eq!(stroke.stroke_width, 3.5);
                match stroke.shape {
                    WhiteboardShapeDto::Rectangle { start, end } => {
                        assert_eq!(start, [10.0, 20.0]);
                        assert_eq!(end, [100.0, 150.0]);
                    }
                    _ => panic!("Expected Rectangle shape"),
                }
            }
            _ => panic!("Expected WhiteboardStroke variant"),
        }

        let server_clear = ServerMessage::WhiteboardClear;
        let srv_clear_json = serde_json::to_string(&server_clear).expect("Failed to serialize Server WhiteboardClear");
        let parsed_clear: ServerMessage = serde_json::from_str(&srv_clear_json).expect("Failed to deserialize Server WhiteboardClear");
        match parsed_clear {
            ServerMessage::WhiteboardClear => {}
            _ => panic!("Expected WhiteboardClear variant"),
        }
    }

    #[test]
    fn test_meeting_policy_dto_defaults_and_serialization() {
        let default_policy = MeetingPolicyDto::default();
        assert!(!default_policy.is_locked);
        assert!(!default_policy.waiting_room_enabled);
        assert!(default_policy.allow_screen_share);
        assert!(default_policy.allow_chat);
        assert!(default_policy.allow_unmute);
        assert!(!default_policy.watermark_enabled);

        let custom_policy = MeetingPolicyDto {
            is_locked: true,
            waiting_room_enabled: true,
            allow_screen_share: false,
            allow_chat: false,
            allow_unmute: false,
            watermark_enabled: true,
        };

        let json = serde_json::to_string(&custom_policy).expect("Failed to serialize policy");
        let parsed: MeetingPolicyDto = serde_json::from_str(&json).expect("Failed to deserialize policy");
        assert_eq!(parsed, custom_policy);

        // Test partial deserialization defaults
        let minimal_json = r#"{}"#;
        let parsed_defaults: MeetingPolicyDto = serde_json::from_str(minimal_json).expect("Failed to deserialize empty policy");
        assert_eq!(parsed_defaults, default_policy);
    }

    #[test]
    fn test_waiting_room_client_messages_serialization() {
        let part_id = Uuid::new_v4();

        let admit_msg = ClientMessage::AdmitParticipant { participant_id: part_id };
        let admit_json = serde_json::to_string(&admit_msg).expect("Serialize AdmitParticipant");
        assert!(admit_json.contains(r#""type":"admit_participant""#));
        assert!(admit_json.contains(&part_id.to_string()));

        let admit_all_msg = ClientMessage::AdmitAll;
        let admit_all_json = serde_json::to_string(&admit_all_msg).expect("Serialize AdmitAll");
        assert_eq!(admit_all_json, r#"{"type":"admit_all"}"#);

        let reject_msg = ClientMessage::RejectParticipant { participant_id: part_id };
        let reject_json = serde_json::to_string(&reject_msg).expect("Serialize RejectParticipant");
        assert!(reject_json.contains(r#""type":"reject_participant""#));
        assert!(reject_json.contains(&part_id.to_string()));

        let policy = MeetingPolicyDto {
            is_locked: false,
            waiting_room_enabled: true,
            allow_screen_share: true,
            allow_chat: true,
            allow_unmute: true,
            watermark_enabled: true,
        };
        let update_policy_msg = ClientMessage::UpdatePolicy { policy: policy.clone() };
        let update_policy_json = serde_json::to_string(&update_policy_msg).expect("Serialize UpdatePolicy");
        assert!(update_policy_json.contains(r#""type":"update_policy""#));
        assert!(update_policy_json.contains(r#""watermark_enabled":true"#));

        let toggle_msg = ClientMessage::ToggleWaitingRoom { enabled: true };
        let toggle_json = serde_json::to_string(&toggle_msg).expect("Serialize ToggleWaitingRoom");
        assert_eq!(toggle_json, r#"{"type":"toggle_waiting_room","enabled":true}"#);
    }

    #[test]
    fn test_waiting_room_server_messages_serialization_and_deserialization() {
        let part_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // 1. WaitingRoomStatus
        let status_msg = ServerMessage::WaitingRoomStatus {
            is_waiting: true,
            message: Some("Please wait, host will admit you shortly.".to_string()),
        };
        let status_json = serde_json::to_string(&status_msg).expect("Serialize WaitingRoomStatus");
        let parsed_status: ServerMessage = serde_json::from_str(&status_json).expect("Deserialize WaitingRoomStatus");
        match parsed_status {
            ServerMessage::WaitingRoomStatus { is_waiting, message } => {
                assert!(is_waiting);
                assert_eq!(message, Some("Please wait, host will admit you shortly.".to_string()));
            }
            _ => panic!("Expected WaitingRoomStatus"),
        }

        // 2. ParticipantWaiting
        let waiting_dto = WaitingParticipantDto {
            participant_id: part_id,
            user_id,
            display_name: "Bob Observer".to_string(),
            email: Some("bob@example.com".to_string()),
            joined_lobby_at: "2026-08-14T20:15:00Z".to_string(),
        };
        let waiting_msg = ServerMessage::ParticipantWaiting { participant: waiting_dto.clone() };
        let waiting_json = serde_json::to_string(&waiting_msg).expect("Serialize ParticipantWaiting");
        let parsed_waiting: ServerMessage = serde_json::from_str(&waiting_json).expect("Deserialize ParticipantWaiting");
        match parsed_waiting {
            ServerMessage::ParticipantWaiting { participant } => {
                assert_eq!(participant.participant_id, part_id);
                assert_eq!(participant.display_name, "Bob Observer");
                assert_eq!(participant.email, Some("bob@example.com".to_string()));
            }
            _ => panic!("Expected ParticipantWaiting"),
        }

        // 3. ParticipantAdmitted
        let admitted_msg = ServerMessage::ParticipantAdmitted { participant_id: part_id };
        let admitted_json = serde_json::to_string(&admitted_msg).expect("Serialize ParticipantAdmitted");
        let parsed_admitted: ServerMessage = serde_json::from_str(&admitted_json).expect("Deserialize ParticipantAdmitted");
        match parsed_admitted {
            ServerMessage::ParticipantAdmitted { participant_id } => assert_eq!(participant_id, part_id),
            _ => panic!("Expected ParticipantAdmitted"),
        }

        // 4. ParticipantRejected
        let rejected_msg = ServerMessage::ParticipantRejected { participant_id: part_id };
        let rejected_json = serde_json::to_string(&rejected_msg).expect("Serialize ParticipantRejected");
        let parsed_rejected: ServerMessage = serde_json::from_str(&rejected_json).expect("Deserialize ParticipantRejected");
        match parsed_rejected {
            ServerMessage::ParticipantRejected { participant_id } => assert_eq!(participant_id, part_id),
            _ => panic!("Expected ParticipantRejected"),
        }

        // 5. MeetingPolicyChanged
        let policy = MeetingPolicyDto {
            is_locked: true,
            waiting_room_enabled: true,
            allow_screen_share: false,
            allow_chat: true,
            allow_unmute: false,
            watermark_enabled: true,
        };
        let policy_msg = ServerMessage::MeetingPolicyChanged { policy: policy.clone() };
        let policy_json = serde_json::to_string(&policy_msg).expect("Serialize MeetingPolicyChanged");
        let parsed_policy: ServerMessage = serde_json::from_str(&policy_json).expect("Deserialize MeetingPolicyChanged");
        match parsed_policy {
            ServerMessage::MeetingPolicyChanged { policy: p } => {
                assert!(p.is_locked);
                assert!(p.waiting_room_enabled);
                assert!(!p.allow_screen_share);
                assert!(p.allow_chat);
                assert!(!p.allow_unmute);
                assert!(p.watermark_enabled);
            }
            _ => panic!("Expected MeetingPolicyChanged"),
        }
    }

    #[test]
    fn test_live_stream_client_messages_serialization() {
        let start_msg = ClientMessage::StartStream {
            rtmp_url: "rtmp://live.youtube.com/app".to_string(),
            stream_key: "live_secret_key_123".to_string(),
        };
        let start_json = serde_json::to_string(&start_msg).expect("Serialize StartStream");
        assert!(start_json.contains(r#""type":"start_stream""#));
        assert!(start_json.contains("rtmp://live.youtube.com/app"));
        assert!(start_json.contains("live_secret_key_123"));

        let parsed_start: ClientMessage = serde_json::from_str(&start_json).expect("Deserialize StartStream");
        match parsed_start {
            ClientMessage::StartStream { rtmp_url, stream_key } => {
                assert_eq!(rtmp_url, "rtmp://live.youtube.com/app");
                assert_eq!(stream_key, "live_secret_key_123");
            }
            _ => panic!("Expected StartStream"),
        }

        let stop_msg = ClientMessage::StopStream;
        let stop_json = serde_json::to_string(&stop_msg).expect("Serialize StopStream");
        assert_eq!(stop_json, r#"{"type":"stop_stream"}"#);

        let parsed_stop: ClientMessage = serde_json::from_str(&stop_json).expect("Deserialize StopStream");
        match parsed_stop {
            ClientMessage::StopStream => {}
            _ => panic!("Expected StopStream"),
        }
    }

    #[test]
    fn test_live_stream_server_messages_serialization_and_deserialization() {
        let live_status_msg = ServerMessage::LiveStreamStatus {
            is_streaming: true,
            rtmp_url: "rtmp://a.rtmp.youtube.com/live2".to_string(),
            status: "live".to_string(),
            started_at: Some("2026-08-14T20:30:00Z".to_string()),
        };
        let live_json = serde_json::to_string(&live_status_msg).expect("Serialize LiveStreamStatus");
        assert!(live_json.contains(r#""type":"live_stream_status""#));
        assert!(live_json.contains(r#""is_streaming":true"#));
        assert!(live_json.contains("rtmp://a.rtmp.youtube.com/live2"));
        assert!(live_json.contains(r#""status":"live""#));

        let parsed_live: ServerMessage = serde_json::from_str(&live_json).expect("Deserialize LiveStreamStatus");
        match parsed_live {
            ServerMessage::LiveStreamStatus { is_streaming, rtmp_url, status, started_at } => {
                assert!(is_streaming);
                assert_eq!(rtmp_url, "rtmp://a.rtmp.youtube.com/live2");
                assert_eq!(status, "live");
                assert_eq!(started_at, Some("2026-08-14T20:30:00Z".to_string()));
            }
            _ => panic!("Expected LiveStreamStatus"),
        }

        let idle_status_msg = ServerMessage::LiveStreamStatus {
            is_streaming: false,
            rtmp_url: "".to_string(),
            status: "idle".to_string(),
            started_at: None,
        };
        let idle_json = serde_json::to_string(&idle_status_msg).expect("Serialize Idle LiveStreamStatus");
        let parsed_idle: ServerMessage = serde_json::from_str(&idle_json).expect("Deserialize Idle LiveStreamStatus");
        match parsed_idle {
            ServerMessage::LiveStreamStatus { is_streaming, rtmp_url, status, started_at } => {
                assert!(!is_streaming);
                assert_eq!(rtmp_url, "");
                assert_eq!(status, "idle");
                assert_eq!(started_at, None);
            }
            _ => panic!("Expected LiveStreamStatus"),
        }
    }

    #[test]
    fn test_caption_messages_serialization_and_deserialization() {
        let part_id = Uuid::new_v4();

        // 1. ClientMessage::SendCaption
        let client_cap = ClientMessage::SendCaption {
            text: "Hello everyone, testing STT streaming captions.".to_string(),
            is_final: true,
            language: "en-US".to_string(),
            timestamp_ms: 1723680000000,
        };
        let client_json = serde_json::to_string(&client_cap).expect("Serialize SendCaption");
        assert!(client_json.contains(r#""type":"send_caption""#));
        assert!(client_json.contains("Hello everyone, testing STT streaming captions."));
        assert!(client_json.contains(r#""is_final":true"#));
        assert!(client_json.contains(r#""language":"en-US""#));

        // Test alias deserialization (caption_chunk)
        let alias_json = r#"{"type":"caption_chunk","text":"Interim speech chunk","is_final":false,"language":"en-US","timestamp_ms":1723680001000}"#;
        let parsed_client: ClientMessage = serde_json::from_str(alias_json).expect("Deserialize SendCaption via alias");
        match parsed_client {
            ClientMessage::SendCaption { text, is_final, language, timestamp_ms } => {
                assert_eq!(text, "Interim speech chunk");
                assert!(!is_final);
                assert_eq!(language, "en-US");
                assert_eq!(timestamp_ms, 1723680001000);
            }
            _ => panic!("Expected SendCaption variant"),
        }

        // 2. ServerMessage::CaptionBroadcast
        let server_cap = ServerMessage::CaptionBroadcast {
            participant_id: part_id,
            speaker_name: "Sarah Conner".to_string(),
            text: "The system is fully operational.".to_string(),
            is_final: true,
            language: "en-US".to_string(),
            timestamp_ms: 1723680002000,
        };
        let srv_json = serde_json::to_string(&server_cap).expect("Serialize CaptionBroadcast");
        assert!(srv_json.contains(r#""type":"caption_broadcast""#));
        assert!(srv_json.contains("Sarah Conner"));
        assert!(srv_json.contains("The system is fully operational."));

        let parsed_srv: ServerMessage = serde_json::from_str(&srv_json).expect("Deserialize CaptionBroadcast");
        match parsed_srv {
            ServerMessage::CaptionBroadcast { participant_id, speaker_name, text, is_final, language, timestamp_ms } => {
                assert_eq!(participant_id, part_id);
                assert_eq!(speaker_name, "Sarah Conner");
                assert_eq!(text, "The system is fully operational.");
                assert!(is_final);
                assert_eq!(language, "en-US");
                assert_eq!(timestamp_ms, 1723680002000);
            }
            _ => panic!("Expected CaptionBroadcast variant"),
        }

        // Test CaptionChunkDto struct
        let chunk_dto = CaptionChunkDto {
            participant_id: part_id,
            speaker_name: "David Developer".to_string(),
            text: "Real-time speech recognition works great!".to_string(),
            is_final: true,
            language: "en-US".to_string(),
            timestamp_ms: 1723680003000,
        };
        let chunk_json = serde_json::to_string(&chunk_dto).expect("Serialize CaptionChunkDto");
        let parsed_dto: CaptionChunkDto = serde_json::from_str(&chunk_json).expect("Deserialize CaptionChunkDto");
        assert_eq!(parsed_dto, chunk_dto);
    }
}

