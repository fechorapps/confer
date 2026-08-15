use std::collections::HashSet;
use std::time::{Duration, Instant};
use egui::{Color32, TextureHandle, Visuals};
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::media::background::BackgroundEffect;
use crate::media::filters::VideoFilter;
use crate::media::{detect_displays, CameraCapturer, DisplayInfo, PickerMode, ScreenCapturer};
use crate::sdk::client::ConferClient;
use crate::sdk::protocol::{ChatMessageDto, ClientMessage, ParticipantState, ServerMessage};
use crate::ui::{lobby, meeting_room};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewState {
    Lobby,
    MeetingRoom,
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

    // Local Media Controls
    pub is_mic_muted: bool,
    pub is_camera_off: bool,
    pub is_screen_sharing: bool,
    pub is_hand_raised: bool,
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

            rtt_ms: 28,
            packet_loss_pct: 0.0,

            client,
            tokio_rt,
        }
    }

    pub fn choose_custom_background(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .set_title("Choose Virtual Background Image")
            .pick_file()
        {
            self.active_background = BackgroundEffect::Custom(path);
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

    pub fn toggle_room_lock(&mut self) {
        self.is_room_locked = !self.is_room_locked;
        // In full flow, dispatches host lock command
    }

    pub fn leave_meeting(&mut self) {
        self.view_state = ViewState::Lobby;
        self.roster.clear();
        self.chat_messages.clear();
        self.active_speaker_ids.clear();
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
                ServerMessage::MeetingLocked { is_locked } => {
                    self.is_room_locked = is_locked;
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
            self.camera_capturer.set_background(self.active_background.clone());
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
                ViewState::MeetingRoom => meeting_room::render_meeting_room(self, ui),
            }
        });
    }
}

fn rand_float() -> f32 {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}
