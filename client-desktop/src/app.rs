use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use egui::{Color32, Pos2, TextureHandle, Visuals};
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::media::background::BackgroundEffect;
use crate::media::filters::VideoFilter;
use crate::media::virtual_background::VirtualBackgroundMode;
use crate::media::{detect_displays, CameraCapturer, DisplayInfo, PickerMode, ScreenCapturer};
use crate::sdk::client::ConferClient;
use crate::sdk::protocol::{
    ChatMessageDto, ClientMessage, MeetingPolicyDto, ParticipantState, PollDto, PollOptionDto,
    ServerMessage, WaitingParticipantDto, WhiteboardColorDto, WhiteboardShapeDto,
    WhiteboardStrokeDto,
};
use crate::ui::whiteboard::{WhiteboardTool, WHITEBOARD_COLORS};
use crate::ui::{lobby, meeting_room, waiting_lobby};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewState {
    Lobby,
    MeetingRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterTab {
    InMeeting,
    WaitingRoom,
}

#[derive(Debug, Clone)]
pub struct ActiveReaction {
    pub emoji: String,
    pub x_offset: f32,
    pub created_at: Instant,
}

pub struct ConferApp {
    pub view_state: ViewState,
    pub server_url: String,

    // User Profile
    pub user_email: String,
    pub user_display_name: String,
    pub my_user_id: Option<Uuid>,
    pub my_participant_id: Option<Uuid>,
    pub my_role: String,

    // Waiting Room & Host Security Policies
    pub is_waiting_in_lobby: bool,
    pub waiting_room_message: Option<String>,
    pub waiting_participants: Vec<WaitingParticipantDto>,
    pub meeting_policy: MeetingPolicyDto,
    pub is_watermark_enabled: bool,
    pub roster_tab: RosterTab,

    // Lobby Inputs
    pub meeting_title_input: String,
    pub join_code_input: String,
    pub mic_test_level: f32,
    pub error_message: Option<String>,

    // In-Meeting State
    pub current_meeting_id: Option<Uuid>,
    pub current_join_code: Option<String>,
    pub room_title: String,
    pub is_room_locked: bool,
    pub roster: Vec<ParticipantState>,
    pub active_speaker_ids: HashSet<Uuid>,
    pub chat_messages: Vec<ChatMessageDto>,
    pub chat_input: String,
    pub unread_chat_count: usize,
    pub active_reactions: Vec<ActiveReaction>,

    // Local Media Controls & Audio/Video AI Engines
    pub is_mic_muted: bool,
    pub is_camera_off: bool,
    pub is_screen_sharing: bool,
    pub is_hand_raised: bool,
    pub is_ai_denoise_enabled: bool,
    pub virtual_bg_mode: VirtualBackgroundMode,
    pub active_filter: VideoFilter,
    pub active_background: BackgroundEffect,

    // Video Engine
    pub camera_capturer: CameraCapturer,
    pub local_video_texture: Option<TextureHandle>,
    pub last_rendered_frame_id: u64,

    // Multi-Display Screen Sharing Engine
    pub available_displays: Vec<DisplayInfo>,
    pub selected_display: Option<DisplayInfo>,
    pub screen_capturer: ScreenCapturer,
    pub screen_share_texture: Option<TextureHandle>,
    pub last_screen_frame_id: u64,

    // Panel Toggles
    pub show_chat: bool,
    pub show_roster: bool,
    pub show_diagnostics: bool,
    pub show_polls: bool,

    // Whiteboard Engine & State
    pub is_whiteboard_active: bool,
    pub whiteboard_strokes: Vec<WhiteboardStrokeDto>,
    pub whiteboard_tool: WhiteboardTool,
    pub whiteboard_color: Color32,
    pub whiteboard_stroke_width: f32,
    pub whiteboard_current_points: Vec<Pos2>,
    pub whiteboard_drag_start: Option<Pos2>,
    pub whiteboard_drag_current: Option<Pos2>,
    pub whiteboard_text_pos: Option<Pos2>,
    pub whiteboard_text_input: String,

    // Polls Engine & State
    pub polls: Vec<PollDto>,
    pub user_poll_votes: HashMap<Uuid, HashSet<usize>>,
    pub poll_selected_options: HashMap<Uuid, HashSet<usize>>,
    pub poll_creating: bool,
    pub poll_create_question: String,
    pub poll_create_options: Vec<String>,
    pub poll_create_multi_choice: bool,
    pub poll_create_anonymous: bool,

    // Metrics
    pub rtt_ms: u32,
    pub packet_loss_pct: f32,

    // Networking
    pub client: ConferClient,
    pub tokio_rt: Runtime,
}

impl ConferApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Runtime::new().expect("Failed to create Tokio runtime");
        let server_url = "http://localhost:5100".to_string();
        let client = ConferClient::new(&server_url);

        Self {
            view_state: ViewState::Lobby,
            server_url,
            user_email: "host@confer.local".to_string(),
            user_display_name: "Host User (Dev)".to_string(),
            my_user_id: None,
            my_participant_id: None,
            my_role: "participant".to_string(),

            is_waiting_in_lobby: false,
            waiting_room_message: None,
            waiting_participants: Vec::new(),
            meeting_policy: MeetingPolicyDto::default(),
            is_watermark_enabled: false,
            roster_tab: RosterTab::InMeeting,

            meeting_title_input: "Daily Standup".to_string(),
            join_code_input: "".to_string(),
            mic_test_level: 0.35,
            error_message: None,

            current_meeting_id: None,
            current_join_code: None,
            room_title: "Confer Meeting".to_string(),
            is_room_locked: false,
            roster: Vec::new(),
            active_speaker_ids: HashSet::new(),
            chat_messages: Vec::new(),
            chat_input: "".to_string(),
            unread_chat_count: 0,
            active_reactions: Vec::new(),

            is_mic_muted: false,
            is_camera_off: false,
            is_screen_sharing: false,
            is_hand_raised: false,
            is_ai_denoise_enabled: true,
            virtual_bg_mode: VirtualBackgroundMode::None,
            active_filter: VideoFilter::None,
            active_background: BackgroundEffect::None,

            camera_capturer: CameraCapturer::new(320, 180),
            local_video_texture: None,
            last_rendered_frame_id: 0,

            available_displays: detect_displays(),
            selected_display: detect_displays().into_iter().next(),
            screen_capturer: ScreenCapturer::new(),
            screen_share_texture: None,
            last_screen_frame_id: 0,

            show_chat: false,
            show_roster: false,
            show_diagnostics: false,
            show_polls: false,

            is_whiteboard_active: false,
            whiteboard_strokes: Vec::new(),
            whiteboard_tool: WhiteboardTool::Pen,
            whiteboard_color: WHITEBOARD_COLORS[0].0,
            whiteboard_stroke_width: 3.0,
            whiteboard_current_points: Vec::new(),
            whiteboard_drag_start: None,
            whiteboard_drag_current: None,
            whiteboard_text_pos: None,
            whiteboard_text_input: "".to_string(),

            polls: Vec::new(),
            user_poll_votes: HashMap::new(),
            poll_selected_options: HashMap::new(),
            poll_creating: false,
            poll_create_question: "".to_string(),
            poll_create_options: vec!["".to_string(), "".to_string()],
            poll_create_multi_choice: false,
            poll_create_anonymous: false,

            rtt_ms: 28,
            packet_loss_pct: 0.0,

            client,
            tokio_rt,
        }
    }

    pub fn toggle_ai_denoise(&mut self) {
        self.is_ai_denoise_enabled = !self.is_ai_denoise_enabled;
    }

    pub fn set_virtual_bg_mode(&mut self, mode: VirtualBackgroundMode) {
        self.virtual_bg_mode = mode.clone();
        self.camera_capturer.set_background(mode);
    }

    pub fn choose_custom_background(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .set_title("Choose Virtual Background Image")
            .pick_file()
        {
            self.virtual_bg_mode = VirtualBackgroundMode::CustomImage(path.clone());
            self.active_background = BackgroundEffect::Custom(path);
            self.camera_capturer.set_background(self.virtual_bg_mode.clone());
        }
    }

    pub fn trigger_create_meeting(&mut self) {
        self.error_message = None;
        let server_url = self.server_url.clone();
        let email = self.user_email.clone();
        let name = self.user_display_name.clone();
        let title = self.meeting_title_input.clone();

        let client = ConferClient::new(&server_url);

        let result = self.tokio_rt.block_on(async {
            let login = client.dev_login(&email, &name).await?;
            let created = client.create_meeting(&title, login.user_id, 50).await?;
            let join = client.join_meeting(created.id, login.user_id, &name).await?;
            Ok::<(Uuid, Uuid, String, String, String, String), crate::sdk::SdkError>((
                login.user_id,
                join.participant_id,
                created.join_code,
                created.title,
                join.role,
                join.room_token,
            ))
        });

        match result {
            Ok((user_id, part_id, join_code, room_title, role, token)) => {
                self.my_user_id = Some(user_id);
                self.my_participant_id = Some(part_id);
                self.current_join_code = Some(join_code);
                self.room_title = room_title;
                self.my_role = role;

                self.client.server_url = self.server_url.clone();
                let mut ws_client = ConferClient::new(&self.server_url);
                let connect_res = self.tokio_rt.block_on(async {
                    ws_client.connect_signaling(&token).await
                });

                if let Err(e) = connect_res {
                    self.error_message = Some(format!("Signaling connection error: {e}"));
                    return;
                }

                self.client = ws_client;
                self.view_state = ViewState::MeetingRoom;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }

    pub fn trigger_join_meeting(&mut self) {
        self.error_message = None;
        if self.join_code_input.trim().is_empty() {
            self.error_message = Some("Please enter a meeting code.".to_string());
            return;
        }

        let server_url = self.server_url.clone();
        let email = self.user_email.clone();
        let name = self.user_display_name.clone();
        let code = self.join_code_input.trim().to_string();

        let client = ConferClient::new(&server_url);

        let result = self.tokio_rt.block_on(async {
            let login = client.dev_login(&email, &name).await?;
            let meeting_info = client.get_meeting_by_code(&code).await?;
            let join = client.join_meeting(meeting_info.id, login.user_id, &name).await?;
            Ok::<(Uuid, Uuid, String, String, String, String), crate::sdk::SdkError>((
                login.user_id,
                join.participant_id,
                meeting_info.join_code,
                meeting_info.title,
                join.role,
                join.room_token,
            ))
        });

        match result {
            Ok((user_id, part_id, join_code, room_title, role, token)) => {
                self.my_user_id = Some(user_id);
                self.my_participant_id = Some(part_id);
                self.current_join_code = Some(join_code);
                self.room_title = room_title;
                self.my_role = role;

                let mut ws_client = ConferClient::new(&self.server_url);
                let connect_res = self.tokio_rt.block_on(async {
                    ws_client.connect_signaling(&token).await
                });

                if let Err(e) = connect_res {
                    self.error_message = Some(format!("Signaling connection error: {e}"));
                    return;
                }

                self.client = ws_client;
                self.view_state = ViewState::MeetingRoom;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }

    pub fn toggle_mic(&mut self) {
        self.is_mic_muted = !self.is_mic_muted;
        self.client.send_message(ClientMessage::SetMute {
            kind: "audio".to_string(),
            muted: self.is_mic_muted,
        });
    }

    pub fn toggle_camera(&mut self) {
        self.is_camera_off = !self.is_camera_off;
        self.client.send_message(ClientMessage::SetMute {
            kind: "video".to_string(),
            muted: self.is_camera_off,
        });
    }

    pub fn start_native_screen_share(&mut self) {
        match self.screen_capturer.start_native() {
            Ok(_) => {
                self.selected_display = None;
                self.is_screen_sharing = true;
                self.client.send_message(ClientMessage::SetMute {
                    kind: "screen_share".to_string(),
                    muted: false,
                });
            }
            Err(e) => {
                tracing::warn!("Native screen capture error: {e}");
            }
        }
    }

    pub fn start_screen_share(&mut self, display: DisplayInfo) {
        self.selected_display = Some(display.clone());
        if let Err(e) = self.screen_capturer.start_display(&display) {
            tracing::warn!("Display screen capture error: {e}");
            return;
        }
        self.is_screen_sharing = true;
        self.client.send_message(ClientMessage::SetMute {
            kind: "screen_share".to_string(),
            muted: false,
        });
    }

    pub fn stop_screen_share(&mut self) {
        self.screen_capturer.stop_capture();
        self.is_screen_sharing = false;
        self.screen_share_texture = None;
        self.last_screen_frame_id = 0;
        self.client.send_message(ClientMessage::SetMute {
            kind: "screen_share".to_string(),
            muted: true,
        });
    }

    #[allow(dead_code)]
    pub fn toggle_screen_share(&mut self) {
        if self.is_screen_sharing {
            self.stop_screen_share();
        } else if self.screen_capturer.picker_mode() == PickerMode::Native {
            self.start_native_screen_share();
        } else {
            let display = self.selected_display.clone()
                .or_else(|| self.available_displays.first().cloned())
                .unwrap_or(DisplayInfo {
                    id: 0,
                    name: ":0.0".to_string(),
                    label: "Primary Display".to_string(),
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                    is_primary: true,
                });
            self.start_screen_share(display);
        }
    }

    pub fn toggle_hand_raise(&mut self) {
        self.is_hand_raised = !self.is_hand_raised;
        // Broadcast custom reaction or state
        self.send_reaction(if self.is_hand_raised { "✋" } else { "👋" });
    }

    pub fn send_reaction(&mut self, emoji: &str) {
        let x_offset = (rand_float() - 0.5) * 200.0;
        self.active_reactions.push(ActiveReaction {
            emoji: emoji.to_string(),
            x_offset,
            created_at: Instant::now(),
        });

        self.client.send_message(ClientMessage::Reaction {
            emoji: emoji.to_string(),
        });
    }

    pub fn send_chat(&mut self) {
        let body = self.chat_input.trim().to_string();
        if body.is_empty() { return; }

        self.client.send_message(ClientMessage::Chat {
            body,
            client_msg_id: Uuid::new_v4(),
        });
        self.chat_input.clear();
    }

    pub fn host_mute_participant(&mut self, participant_id: Uuid) {
        self.client.send_message(ClientMessage::HostAction {
            action: "mute".to_string(),
            target_participant_id: participant_id,
        });
    }

    pub fn host_kick_participant(&mut self, participant_id: Uuid) {
        self.client.send_message(ClientMessage::HostAction {
            action: "kick".to_string(),
            target_participant_id: participant_id,
        });
    }

    pub fn admit_participant(&mut self, participant_id: Uuid) {
        self.client.send_message(ClientMessage::AdmitParticipant { participant_id });
        self.waiting_participants.retain(|p| p.participant_id != participant_id);
    }

    pub fn admit_all_waiting(&mut self) {
        self.client.send_message(ClientMessage::AdmitAll);
        self.waiting_participants.clear();
    }

    pub fn reject_participant(&mut self, participant_id: Uuid) {
        self.client.send_message(ClientMessage::RejectParticipant { participant_id });
        self.waiting_participants.retain(|p| p.participant_id != participant_id);
    }

    pub fn update_meeting_policy(&mut self, policy: MeetingPolicyDto) {
        self.is_room_locked = policy.is_locked;
        self.is_watermark_enabled = policy.watermark_enabled;
        self.meeting_policy = policy.clone();
        self.client.send_message(ClientMessage::UpdatePolicy { policy });
    }

    pub fn toggle_waiting_room(&mut self, enabled: bool) {
        let mut policy = self.meeting_policy.clone();
        policy.waiting_room_enabled = enabled;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_room_lock(&mut self) {
        let mut policy = self.meeting_policy.clone();
        policy.is_locked = !policy.is_locked;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_allow_screen_share(&mut self) {
        let mut policy = self.meeting_policy.clone();
        policy.allow_screen_share = !policy.allow_screen_share;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_allow_chat(&mut self) {
        let mut policy = self.meeting_policy.clone();
        policy.allow_chat = !policy.allow_chat;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_allow_unmute(&mut self) {
        let mut policy = self.meeting_policy.clone();
        policy.allow_unmute = !policy.allow_unmute;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_watermark(&mut self) {
        let mut policy = self.meeting_policy.clone();
        policy.watermark_enabled = !policy.watermark_enabled;
        self.update_meeting_policy(policy);
    }

    pub fn toggle_whiteboard(&mut self) {
        self.is_whiteboard_active = !self.is_whiteboard_active;
        if self.is_whiteboard_active {
            if self.is_screen_sharing {
                self.stop_screen_share();
            }
        }
    }

    pub fn toggle_polls(&mut self) {
        self.show_polls = !self.show_polls;
        if self.show_polls {
            self.show_chat = false;
            self.show_roster = false;
        }
    }

    pub fn add_whiteboard_stroke(&mut self, stroke: WhiteboardStrokeDto) {
        self.client.send_message(ClientMessage::WhiteboardStroke {
            stroke: stroke.clone(),
        });
        self.whiteboard_strokes.push(stroke);
    }

    pub fn commit_whiteboard_text(&mut self, pos: Pos2) {
        let text = self.whiteboard_text_input.trim().to_string();
        if text.is_empty() { return; }

        let stroke = WhiteboardStrokeDto {
            id: Uuid::new_v4(),
            participant_id: self.my_participant_id.unwrap_or(Uuid::nil()),
            shape: WhiteboardShapeDto::Text {
                pos: [pos.x, pos.y],
                text,
                font_size: (self.whiteboard_stroke_width * 5.0).clamp(12.0, 48.0),
            },
            color: WhiteboardColorDto::new(
                self.whiteboard_color.r(),
                self.whiteboard_color.g(),
                self.whiteboard_color.b(),
                self.whiteboard_color.a(),
            ),
            stroke_width: self.whiteboard_stroke_width,
        };
        self.add_whiteboard_stroke(stroke);
        self.whiteboard_text_input.clear();
        self.whiteboard_text_pos = None;
    }

    pub fn undo_whiteboard_stroke(&mut self) {
        if let Some(pos) = self.whiteboard_strokes.iter().rposition(|s| Some(s.participant_id) == self.my_participant_id) {
            self.whiteboard_strokes.remove(pos);
        } else {
            self.whiteboard_strokes.pop();
        }
    }

    pub fn clear_whiteboard(&mut self) {
        self.whiteboard_strokes.clear();
        self.client.send_message(ClientMessage::WhiteboardClear);
    }

    pub fn trigger_create_poll(&mut self) {
        let question = self.poll_create_question.trim().to_string();
        if question.is_empty() { return; }

        let options: Vec<String> = self.poll_create_options
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if options.len() < 2 { return; }

        let poll_id = Uuid::new_v4();
        let creator_id = self.my_user_id.unwrap_or(Uuid::nil());
        let creator_name = self.user_display_name.clone();

        let poll_dto = PollDto {
            id: poll_id,
            creator_id,
            creator_name,
            question: question.clone(),
            options: options.iter().enumerate().map(|(id, text)| PollOptionDto {
                id,
                text: text.clone(),
                vote_count: 0,
                voter_ids: Vec::new(),
            }).collect(),
            multi_choice: self.poll_create_multi_choice,
            is_anonymous: self.poll_create_anonymous,
            is_closed: false,
            total_votes: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.client.send_message(ClientMessage::CreatePoll {
            poll_id,
            question,
            options,
            multi_choice: self.poll_create_multi_choice,
            is_anonymous: self.poll_create_anonymous,
        });

        self.polls.push(poll_dto);
        self.poll_creating = false;
        self.poll_create_question.clear();
        self.poll_create_options = vec!["".to_string(), "".to_string()];
        self.poll_create_multi_choice = false;
        self.poll_create_anonymous = false;
    }

    pub fn vote_poll(&mut self, poll_id: Uuid, selected_options: Vec<usize>) {
        if selected_options.is_empty() { return; }

        let user_id = self.my_user_id.unwrap_or(Uuid::nil());

        // Update local poll state immediately
        if let Some(poll) = self.polls.iter_mut().find(|p| p.id == poll_id) {
            for &opt_id in &selected_options {
                if let Some(opt) = poll.options.iter_mut().find(|o| o.id == opt_id) {
                    opt.vote_count += 1;
                    if !poll.is_anonymous && !opt.voter_ids.contains(&user_id) {
                        opt.voter_ids.push(user_id);
                    }
                }
            }
            poll.total_votes += 1;
        }

        let votes_set: HashSet<usize> = selected_options.iter().copied().collect();
        self.user_poll_votes.insert(poll_id, votes_set);
        self.poll_selected_options.remove(&poll_id);

        self.client.send_message(ClientMessage::VotePoll {
            poll_id,
            selected_options,
        });
    }

    pub fn close_poll(&mut self, poll_id: Uuid) {
        if let Some(poll) = self.polls.iter_mut().find(|p| p.id == poll_id) {
            poll.is_closed = true;
        }
        self.client.send_message(ClientMessage::ClosePoll { poll_id });
    }

    pub fn leave_meeting(&mut self) {
        self.view_state = ViewState::Lobby;
        self.is_waiting_in_lobby = false;
        self.waiting_room_message = None;
        self.waiting_participants.clear();
        self.meeting_policy = MeetingPolicyDto::default();
        self.is_watermark_enabled = false;
        self.roster_tab = RosterTab::InMeeting;
        self.roster.clear();
        self.chat_messages.clear();
        self.active_speaker_ids.clear();
        self.whiteboard_strokes.clear();
        self.polls.clear();
        self.user_poll_votes.clear();
        self.is_whiteboard_active = false;
        self.show_polls = false;
        self.current_meeting_id = None;
        self.current_join_code = None;
    }

    fn poll_incoming_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(rx) = &mut self.client.incoming_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        for msg in messages {
            match msg {
                ServerMessage::Joined { roster, .. } => {
                    self.roster = roster.into_iter()
                        .filter(|p| Some(p.participant_id) != self.my_participant_id)
                        .collect();
                }
                ServerMessage::ParticipantJoined { participant } => {
                    if Some(participant.participant_id) != self.my_participant_id {
                        self.roster.retain(|p| p.participant_id != participant.participant_id);
                        self.roster.push(participant);
                    }
                }
                ServerMessage::ParticipantLeft { participant_id, .. } => {
                    self.roster.retain(|p| p.participant_id != participant_id);
                    self.active_speaker_ids.remove(&participant_id);
                }
                ServerMessage::ParticipantMuteChanged { participant_id, kind, muted } => {
                    if let Some(p) = self.roster.iter_mut().find(|p| p.participant_id == participant_id) {
                        if kind == "audio" { p.is_audio_muted = muted; }
                        if kind == "video" { p.is_video_muted = muted; }
                        if kind == "screen_share" { p.is_screen_sharing = !muted; }
                    }
                }
                ServerMessage::ActiveSpeakers { ranked } => {
                    self.active_speaker_ids.clear();
                    for spk in ranked {
                        self.active_speaker_ids.insert(spk.participant_id);
                    }
                }
                ServerMessage::Chat { id, from_id, from_name, body, sent_at } => {
                    self.chat_messages.push(ChatMessageDto {
                        id,
                        from_id,
                        from_name,
                        body,
                        sent_at: sent_at[11..16.min(sent_at.len())].to_string(), // HH:MM
                    });
                    if !self.show_chat {
                        self.unread_chat_count += 1;
                    }
                }
                ServerMessage::Reaction { emoji, .. } => {
                    let x_offset = (rand_float() - 0.5) * 200.0;
                    self.active_reactions.push(ActiveReaction {
                        emoji,
                        x_offset,
                        created_at: Instant::now(),
                    });
                }
                ServerMessage::PollCreated { poll } => {
                    if !self.polls.iter().any(|p| p.id == poll.id) {
                        self.polls.push(poll);
                    }
                }
                ServerMessage::PollUpdated { poll } => {
                    if let Some(existing) = self.polls.iter_mut().find(|p| p.id == poll.id) {
                        *existing = poll;
                    } else {
                        self.polls.push(poll);
                    }
                }
                ServerMessage::PollClosed { poll_id } => {
                    if let Some(poll) = self.polls.iter_mut().find(|p| p.id == poll_id) {
                        poll.is_closed = true;
                    }
                }
                ServerMessage::WhiteboardStroke { stroke } => {
                    if !self.whiteboard_strokes.iter().any(|s| s.id == stroke.id) {
                        self.whiteboard_strokes.push(stroke);
                    }
                }
                ServerMessage::WhiteboardClear => {
                    self.whiteboard_strokes.clear();
                }
                ServerMessage::MeetingLocked { is_locked } => {
                    self.is_room_locked = is_locked;
                    self.meeting_policy.is_locked = is_locked;
                }
                ServerMessage::WaitingRoomStatus { is_waiting, message } => {
                    self.is_waiting_in_lobby = is_waiting;
                    self.waiting_room_message = message;
                }
                ServerMessage::ParticipantWaiting { participant } => {
                    if !self.waiting_participants.iter().any(|p| p.participant_id == participant.participant_id) {
                        self.waiting_participants.push(participant);
                    }
                }
                ServerMessage::ParticipantAdmitted { participant_id } => {
                    if Some(participant_id) == self.my_participant_id {
                        self.is_waiting_in_lobby = false;
                        self.waiting_room_message = None;
                    }
                    self.waiting_participants.retain(|p| p.participant_id != participant_id);
                }
                ServerMessage::ParticipantRejected { participant_id } => {
                    if Some(participant_id) == self.my_participant_id {
                        self.leave_meeting();
                        self.error_message = Some("The host rejected your request to join the meeting.".to_string());
                    }
                    self.waiting_participants.retain(|p| p.participant_id != participant_id);
                }
                ServerMessage::MeetingPolicyChanged { policy } => {
                    self.is_room_locked = policy.is_locked;
                    self.is_watermark_enabled = policy.watermark_enabled;
                    self.meeting_policy = policy;
                }
                ServerMessage::MeetingEnded { .. } => {
                    self.leave_meeting();
                }
                ServerMessage::Pong { .. } => {
                    // Update RTT
                    self.rtt_ms = 25;
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for ConferApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Dark theme matching Confer design system (Obsidian & Deep Zinc)
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(11, 12, 14);
        visuals.window_fill = Color32::from_rgb(18, 20, 23);
        visuals.selection.bg_fill = Color32::from_rgb(2, 132, 199);
        ctx.set_visuals(visuals);

        self.poll_incoming_messages();

        // Update local live camera frame texture with zero redundant GPU uploads
        if !self.is_camera_off {
            self.camera_capturer.set_filter(self.active_filter);
            self.camera_capturer.set_background(self.virtual_bg_mode.clone());
            if let Some((frame_id, frame)) = self.camera_capturer.get_latest_frame_if_newer(self.last_rendered_frame_id) {
                self.local_video_texture = Some(ctx.load_texture("local_camera_feed", frame, egui::TextureOptions::LINEAR));
                self.last_rendered_frame_id = frame_id;
            }
        } else {
            self.local_video_texture = None;
            self.last_rendered_frame_id = 0;
        }

        // Update real-time screen share texture
        if self.is_screen_sharing {
            // Backends that negotiate off-thread (e.g. the portal backend, so
            // its interactive picker dialog never blocks this UI thread) can
            // only report a failed start asynchronously; pick that up here.
            if let Some(err) = self.screen_capturer.take_error() {
                tracing::warn!("Screen capture failed: {err}");
                self.error_message = Some(format!("Screen share failed: {err}"));
                self.is_screen_sharing = false;
                self.screen_share_texture = None;
                self.last_screen_frame_id = 0;
                self.client.send_message(ClientMessage::SetMute {
                    kind: "screen_share".to_string(),
                    muted: true,
                });
            } else if let Some((frame_id, frame)) = self.screen_capturer.get_latest_frame_if_newer(self.last_screen_frame_id) {
                self.screen_share_texture = Some(ctx.load_texture("screen_share_feed", frame, egui::TextureOptions::LINEAR));
                self.last_screen_frame_id = frame_id;
            }
        } else {
            self.screen_share_texture = None;
            self.last_screen_frame_id = 0;
        }

        // Request continuous repaint for smooth 60fps animations
        ctx.request_repaint_after(Duration::from_millis(16));

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.view_state {
                ViewState::Lobby => lobby::render_lobby(self, ui),
                ViewState::MeetingRoom => {
                    if self.is_waiting_in_lobby {
                        waiting_lobby::render_waiting_lobby(self, ui);
                    } else {
                        meeting_room::render_meeting_room(self, ui);
                    }
                }
            }
        });
    }
}

fn rand_float() -> f32 {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}
