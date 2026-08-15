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
}
