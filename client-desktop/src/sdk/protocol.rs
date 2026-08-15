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
}
