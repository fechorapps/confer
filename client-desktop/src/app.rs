use egui::{Color32, Pos2, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::media::filters::VideoFilter;
use crate::media::virtual_background::VirtualBackgroundMode;
use crate::media::{
    detect_displays, AudioPipeline, CameraCapturer, CandidateTarget, DisplayInfo, PickerMode,
    RtcEngine, RtcEvent, ScreenCapturer, VideoDecoder, VideoEncoder,
};
use crate::sdk::client::ConferClient;
use crate::sdk::protocol::{
    CaptionChunkDto, ChatMessageDto, ClientMessage, MeetingPolicyDto, ParticipantState, PollDto,
    PollOptionDto, ServerMessage, WaitingParticipantDto, WhiteboardColorDto, WhiteboardShapeDto,
    WhiteboardStrokeDto,
};

use crate::ui::whiteboard::{WhiteboardTool, WHITEBOARD_COLORS};
use crate::ui::{lobby, meeting_room, waiting_lobby};
use crate::ui::theme::Theme;

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

/// Which async connect flow to run: create a new meeting or join by code.
pub enum ConnectAction {
    Create { title: String },
    Join { code: String },
}

/// Result of the full async login → create/join → signaling-connect flow,
/// delivered back to the UI thread through `connect_rx`.
pub struct ConnectSuccess {
    pub user_id: Uuid,
    pub participant_id: Uuid,
    pub join_code: String,
    pub room_title: String,
    pub role: String,
    pub client: ConferClient,
    pub rtc_engine: Option<Arc<tokio::sync::Mutex<RtcEngine>>>,
    pub audio_pipeline: Option<AudioPipeline>,
    pub video_encoder: Option<VideoEncoder>,
    pub local_frame_tx: Option<mpsc::Sender<Vec<u8>>>,
    pub remote_frame_rx: Option<mpsc::UnboundedReceiver<(Uuid, egui::ColorImage)>>,
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
    /// True while the async login → create/join → signaling-connect flow runs.
    pub is_connecting: bool,

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
    pub is_captions_enabled: bool,
    pub active_captions: Vec<CaptionChunkDto>,

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
    pub whiteboard_zoom: f32,
    pub whiteboard_pan: egui::Vec2,
    pub show_stage_filmstrip: bool,

    // Polls Engine & State
    pub polls: Vec<PollDto>,
    pub user_poll_votes: HashMap<Uuid, HashSet<usize>>,
    pub poll_selected_options: HashMap<Uuid, HashSet<usize>>,
    pub poll_creating: bool,
    pub poll_create_question: String,
    pub poll_create_options: Vec<String>,
    pub poll_create_multi_choice: bool,
    pub poll_create_anonymous: bool,

    // Safety & Ergonomics State
    pub show_leave_confirmation: bool,
    pub show_whiteboard_clear_confirmation: bool,
    pub kick_confirmation_target: Option<(Uuid, String)>,
    pub is_push_to_talk_active: bool,

    // Metrics
    pub rtt_ms: u32,
    pub packet_loss_pct: f32,

    // Networking
    pub client: ConferClient,
    pub tokio_rt: Runtime,
    /// Receives the outcome of the spawned connect flow (drained in `update()`).
    pub connect_rx: Option<mpsc::UnboundedReceiver<Result<ConnectSuccess, String>>>,
    /// WebRTC engine, created after the signaling connection is established.
    pub rtc_engine: Option<Arc<tokio::sync::Mutex<RtcEngine>>>,
    /// Real-time microphone capture and remote audio playback pipeline.
    pub audio_pipeline: Option<AudioPipeline>,
    /// Real-time VP8 video encoder pipeline (Step 4).
    pub video_encoder: Option<VideoEncoder>,
    /// Channel to send raw RGB camera/screen frames to the video encoder.
    pub local_frame_tx: Option<mpsc::Sender<Vec<u8>>>,
    /// Decoded remote video frames receiver (Step 5).
    pub remote_frame_rx: Option<mpsc::UnboundedReceiver<(Uuid, egui::ColorImage)>>,
    /// Active GPU textures for remote participant video feeds.
    pub remote_video_textures: HashMap<Uuid, TextureHandle>,

    /// Timestamp when join code was copied to clipboard for ephemeral toast feedback
    pub code_copied_time: Option<f64>,

    /// Selected cockpit tab in the Pre-Flight Lobby (Host vs Join)
    pub lobby_action_tab: LobbyActionTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LobbyActionTab {
    #[default]
    Host,
    Join,
}

impl ConferApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = Runtime::new().expect("Failed to create Tokio runtime");
        let server_url = "http://localhost:5100".to_string();
        let client = ConferClient::new(&server_url);
        let available_displays = detect_displays();
        let selected_display = available_displays.first().cloned();

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
            is_connecting: false,

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

            camera_capturer: CameraCapturer::new(320, 180),
            local_video_texture: None,
            last_rendered_frame_id: 0,

            available_displays,
            selected_display,
            screen_capturer: ScreenCapturer::new(),
            screen_share_texture: None,
            last_screen_frame_id: 0,

            show_chat: false,
            show_roster: false,
            show_diagnostics: false,
            show_polls: false,
            is_captions_enabled: false,
            active_captions: Vec::new(),

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
            whiteboard_zoom: 1.0,
            whiteboard_pan: egui::Vec2::ZERO,
            show_stage_filmstrip: true,

            polls: Vec::new(),
            user_poll_votes: HashMap::new(),
            poll_selected_options: HashMap::new(),
            poll_creating: false,
            poll_create_question: "".to_string(),
            poll_create_options: vec!["".to_string(), "".to_string()],
            poll_create_multi_choice: false,
            poll_create_anonymous: false,

            show_leave_confirmation: false,
            show_whiteboard_clear_confirmation: false,
            kick_confirmation_target: None,
            is_push_to_talk_active: false,

            // 0 = no RTT measurement yet (never report a fabricated value)
            rtt_ms: 0,
            packet_loss_pct: 0.0,

            client,
            tokio_rt,
            connect_rx: None,
            rtc_engine: None,
            audio_pipeline: None,
            video_encoder: None,
            local_frame_tx: None,
            remote_frame_rx: None,
            remote_video_textures: HashMap::new(),
            code_copied_time: None,
            lobby_action_tab: LobbyActionTab::default(),
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
            self.virtual_bg_mode = VirtualBackgroundMode::CustomImage(path);
            self.camera_capturer
                .set_background(self.virtual_bg_mode.clone());
        }
    }

    pub fn trigger_create_meeting(&mut self) {
        let title = self.meeting_title_input.clone();
        self.spawn_meeting_connect(ConnectAction::Create { title });
    }

    pub fn trigger_join_meeting(&mut self) {
        self.error_message = None;
        let code = self.join_code_input.trim().to_string();
        if code.is_empty() {
            self.error_message = Some("Please enter a meeting code.".to_string());
            return;
        }
        self.spawn_meeting_connect(ConnectAction::Join { code });
    }

    /// Spawns the full login → create/join → signaling-connect flow on the
    /// Tokio runtime so the UI thread never blocks; the outcome arrives
    /// through `connect_rx` and is applied in `poll_connect_result`.
    fn spawn_meeting_connect(&mut self, action: ConnectAction) {
        self.error_message = None;
        if self.is_connecting {
            return;
        }
        self.is_connecting = true;

        let server_url = self.server_url.clone();
        let email = self.user_email.clone();
        let name = self.user_display_name.clone();

        let (tx, rx) = mpsc::unbounded_channel();
        self.connect_rx = Some(rx);

        let rt_handle = self.tokio_rt.handle().clone();

        self.tokio_rt.spawn(async move {
            let outcome: Result<ConnectSuccess, String> = async {
                let client = ConferClient::new(&server_url);
                let login = client
                    .dev_login(&email, &name)
                    .await
                    .map_err(|e| e.to_string())?;

                let (meeting_id, join_code, room_title) = match action {
                    ConnectAction::Create { title } => {
                        let created = client
                            .create_meeting(&title, login.user_id, 50)
                            .await
                            .map_err(|e| e.to_string())?;
                        (created.id, created.join_code, created.title)
                    }
                    ConnectAction::Join { code } => {
                        let meeting_info = client
                            .get_meeting_by_code(&code)
                            .await
                            .map_err(|e| e.to_string())?;
                        (meeting_info.id, meeting_info.join_code, meeting_info.title)
                    }
                };

                let join = client
                    .join_meeting(meeting_id, login.user_id, &name)
                    .await
                    .map_err(|e| e.to_string())?;

                let mut ws_client = ConferClient::new(&server_url);
                ws_client
                    .connect_signaling(&join.room_token)
                    .await
                    .map_err(|e| format!("Signaling connection error: {e}"))?;

                let outgoing_tx = ws_client
                    .outgoing_tx
                    .as_ref()
                    .ok_or_else(|| "Signaling connection has no outgoing channel".to_string())?
                    .clone();

                let (rtc_tx, mut rtc_rx) = mpsc::channel::<RtcEvent>(64);
                let rtc_engine = RtcEngine::new(join.ice_servers, rtc_tx)
                    .await
                    .map_err(|e| format!("Failed to create RTC engine: {e}"))?;

                let audio_track = rtc_engine.audio_track();
                let audio_pipeline = AudioPipeline::new(audio_track)
                    .await
                    .map_err(|e| format!("Failed to create audio pipeline: {e}"))?;
                let remote_sample_tx = audio_pipeline.remote_sample_tx();

                let video_track = rtc_engine.video_track();
                let (local_frame_tx, local_frame_rx) = mpsc::channel::<Vec<u8>>(8);
                let video_encoder =
                    VideoEncoder::start(320, 180, 30, 500, video_track, local_frame_rx).ok();

                let (remote_frame_tx, remote_frame_rx) =
                    mpsc::unbounded_channel::<(Uuid, egui::ColorImage)>();

                let rtc_engine = Arc::new(tokio::sync::Mutex::new(rtc_engine));

                rt_handle.spawn(async move {
                    while let Some(event) = rtc_rx.recv().await {
                        match event {
                            RtcEvent::IceCandidate {
                                target,
                                candidate,
                                sdp_mid,
                                sdp_mline_index,
                                username_fragment,
                            } => {
                                let msg = ClientMessage::IceCandidate {
                                    target: target.to_string(),
                                    candidate,
                                    sdp_mid,
                                    sdp_mline_index,
                                    username_fragment,
                                };
                                if let Err(e) = outgoing_tx.try_send(msg) {
                                    tracing::warn!("failed to enqueue local ICE candidate: {e}");
                                }
                            }
                            RtcEvent::RemoteTrack {
                                publisher_id,
                                kind,
                                track,
                            } => {
                                if kind == "audio" {
                                    AudioPipeline::start_remote_decode(track, remote_sample_tx.clone());
                                } else if kind == "video" {
                                    tracing::info!(
                                        "remote video track ready (step 5): publisher_id={publisher_id}"
                                    );
                                    let dec_tx = remote_frame_tx.clone();
                                    let _ = VideoDecoder::start(track, dec_tx, publisher_id);
                                }
                            }
                        }
                    }
                });

                Ok(ConnectSuccess {
                    user_id: login.user_id,
                    participant_id: join.participant_id,
                    join_code,
                    room_title,
                    role: join.role,
                    client: ws_client,
                    rtc_engine: Some(rtc_engine),
                    audio_pipeline: Some(audio_pipeline),
                    video_encoder,
                    local_frame_tx: Some(local_frame_tx),
                    remote_frame_rx: Some(remote_frame_rx),
                })
            }
            .await;

            let _ = tx.send(outcome);
        });
    }

    /// Drains the connect-flow outcome channel (called from `update()`).
    fn poll_connect_result(&mut self) {
        let Some(rx) = &mut self.connect_rx else {
            return;
        };
        let outcome = match rx.try_recv() {
            Ok(outcome) => outcome,
            Err(mpsc::error::TryRecvError::Empty) => return,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err("Connection task terminated unexpectedly.".to_string())
            }
        };

        self.connect_rx = None;
        self.is_connecting = false;

        match outcome {
            Ok(success) => {
                self.my_user_id = Some(success.user_id);
                self.my_participant_id = Some(success.participant_id);
                self.current_join_code = Some(success.join_code);
                self.room_title = success.room_title;
                self.my_role = success.role;
                self.client = success.client;
                self.rtc_engine = success.rtc_engine;
                self.audio_pipeline = success.audio_pipeline;
                self.video_encoder = success.video_encoder;
                self.local_frame_tx = success.local_frame_tx;
                self.remote_frame_rx = success.remote_frame_rx;
                self.view_state = ViewState::MeetingRoom;
            }
            Err(message) => {
                self.error_message = Some(message);
            }
        }
    }

    pub fn toggle_mic(&mut self) {
        self.is_mic_muted = !self.is_mic_muted;
        if let Some(pipeline) = &self.audio_pipeline {
            pipeline.set_muted(self.is_mic_muted);
        }
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
            let display = self
                .selected_display
                .clone()
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
        self.spawn_reaction(emoji.to_string());

        self.client.send_message(ClientMessage::Reaction {
            emoji: emoji.to_string(),
        });
    }

    /// Pushes a locally-rendered floating reaction animation.
    fn spawn_reaction(&mut self, emoji: String) {
        let x_offset = (rand_float() - 0.5) * 200.0;
        self.active_reactions.push(ActiveReaction {
            emoji,
            x_offset,
            created_at: Instant::now(),
        });
    }

    pub fn send_chat(&mut self) {
        let body = self.chat_input.trim().to_string();
        if body.is_empty() {
            return;
        }

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
        self.client
            .send_message(ClientMessage::AdmitParticipant { participant_id });
        self.waiting_participants
            .retain(|p| p.participant_id != participant_id);
    }

    pub fn admit_all_waiting(&mut self) {
        self.client.send_message(ClientMessage::AdmitAll);
        self.waiting_participants.clear();
    }

    pub fn reject_participant(&mut self, participant_id: Uuid) {
        self.client
            .send_message(ClientMessage::RejectParticipant { participant_id });
        self.waiting_participants
            .retain(|p| p.participant_id != participant_id);
    }

    pub fn update_meeting_policy(&mut self, policy: MeetingPolicyDto) {
        self.is_room_locked = policy.is_locked;
        self.is_watermark_enabled = policy.watermark_enabled;
        self.meeting_policy = policy.clone();
        self.client
            .send_message(ClientMessage::UpdatePolicy { policy });
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
        if self.is_whiteboard_active && self.is_screen_sharing {
            self.stop_screen_share();
        }
    }

    pub fn toggle_polls(&mut self) {
        self.show_polls = !self.show_polls;
        if self.show_polls {
            self.show_chat = false;
            self.show_roster = false;
        }
    }

    pub fn toggle_captions(&mut self) {
        self.is_captions_enabled = !self.is_captions_enabled;
    }

    #[allow(dead_code)]
    pub fn send_caption(&mut self, text: &str, is_final: bool) {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.client.send_message(ClientMessage::SendCaption {
            text: text.to_string(),
            is_final,
            language: "en-US".to_string(),
            timestamp_ms,
        });
    }

    pub fn add_caption_chunk(&mut self, chunk: CaptionChunkDto) {
        if let Some(existing) = self
            .active_captions
            .iter_mut()
            .rev()
            .find(|c| c.participant_id == chunk.participant_id && !c.is_final)
        {
            existing.text = chunk.text;
            existing.is_final = chunk.is_final;
            existing.language = chunk.language;
            existing.timestamp_ms = chunk.timestamp_ms;
        } else {
            self.active_captions.push(chunk);
            if self.active_captions.len() > 10 {
                self.active_captions.remove(0);
            }
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
        if text.is_empty() {
            return;
        }

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
        if let Some(pos) = self
            .whiteboard_strokes
            .iter()
            .rposition(|s| Some(s.participant_id) == self.my_participant_id)
        {
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
        if question.is_empty() {
            return;
        }

        let options: Vec<String> = self
            .poll_create_options
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if options.len() < 2 {
            return;
        }

        let poll_id = Uuid::new_v4();
        let creator_id = self.my_user_id.unwrap_or(Uuid::nil());
        let creator_name = self.user_display_name.clone();

        let poll_dto = PollDto {
            id: poll_id,
            creator_id,
            creator_name,
            question: question.clone(),
            options: options
                .iter()
                .enumerate()
                .map(|(id, text)| PollOptionDto {
                    id,
                    text: text.clone(),
                    vote_count: 0,
                    voter_ids: Vec::new(),
                })
                .collect(),
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
        if selected_options.is_empty() {
            return;
        }

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
        self.client
            .send_message(ClientMessage::ClosePoll { poll_id });
    }

    pub fn leave_meeting(&mut self) {
        // Drop the audio and video pipelines and WebRTC engine so PeerConnections release
        // their sockets and media streams are stopped. The on-going RtcEvent
        // task will exit when its receiver closes.
        self.audio_pipeline = None;
        self.video_encoder = None;
        self.local_frame_tx = None;
        self.remote_frame_rx = None;
        self.remote_video_textures.clear();
        self.rtc_engine = None;

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
        self.active_captions.clear();
        self.is_whiteboard_active = false;
        self.show_polls = false;
        self.show_leave_confirmation = false;
        self.kick_confirmation_target = None;
        self.is_push_to_talk_active = false;
        self.current_meeting_id = None;
        self.current_join_code = None;
        self.code_copied_time = None;
    }

    pub fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        // If a text input has active focus, do not intercept typing
        if ctx.wants_keyboard_input() {
            return;
        }

        let input = ctx.input(|i| i.clone());
        let ctrl_or_cmd = input.modifiers.command || input.modifiers.ctrl;

        // Waiting Lobby shortcuts (Mic/Cam test and Escape dismiss)
        if self.is_waiting_in_lobby {
            if input.key_pressed(egui::Key::Escape) {
                self.show_leave_confirmation = false;
            }
            if ctrl_or_cmd {
                if input.key_pressed(egui::Key::D) {
                    self.toggle_mic();
                }
                if input.key_pressed(egui::Key::E) {
                    self.toggle_camera();
                }
            }
            return;
        }

        // Pre-Flight Lobby Shortcuts
        if self.view_state == ViewState::Lobby {
            if ctrl_or_cmd {
                // Ctrl + D -> Toggle Mic
                if input.key_pressed(egui::Key::D) {
                    self.toggle_mic();
                }
                // Ctrl + E -> Toggle Camera
                if input.key_pressed(egui::Key::E) {
                    self.toggle_camera();
                }
                // Ctrl + Enter -> Start Meeting Now
                if input.key_pressed(egui::Key::Enter) && !self.is_connecting {
                    self.trigger_create_meeting();
                }
            }
            return;
        }

        if self.view_state != ViewState::MeetingRoom {
            return;
        }

        // In-Meeting Shortcuts:
        // 1. Push-to-Talk (Hold Spacebar)
        if input.key_down(egui::Key::Space) {
            if self.is_mic_muted && !self.is_push_to_talk_active {
                self.is_push_to_talk_active = true;
                self.client.send_message(ClientMessage::SetMute {
                    kind: "audio".to_string(),
                    muted: false,
                });
            }
        } else if self.is_push_to_talk_active {
            self.is_push_to_talk_active = false;
            self.client.send_message(ClientMessage::SetMute {
                kind: "audio".to_string(),
                muted: true,
            });
        }

        // 2. Ctrl/Cmd Mnemonic Hotkeys
        if ctrl_or_cmd {
            // Ctrl + D -> Toggle Mic
            if input.key_pressed(egui::Key::D) {
                self.toggle_mic();
            }
            // Ctrl + E -> Toggle Camera
            if input.key_pressed(egui::Key::E) {
                self.toggle_camera();
            }
            // Ctrl + Shift + S -> Toggle Screen Share
            if input.modifiers.shift && input.key_pressed(egui::Key::S) {
                self.toggle_screen_share();
            }
            // Ctrl + Shift + C -> Toggle Captions
            if input.modifiers.shift && input.key_pressed(egui::Key::C) {
                self.toggle_captions();
            }
            // Ctrl + Shift + W -> Toggle Whiteboard
            if input.modifiers.shift && input.key_pressed(egui::Key::W) {
                self.toggle_whiteboard();
            }
            // Ctrl + Shift + P -> Toggle Polls
            if input.modifiers.shift && input.key_pressed(egui::Key::P) {
                self.toggle_polls();
            }
            // Ctrl + Shift + M -> Toggle Chat
            if input.modifiers.shift && input.key_pressed(egui::Key::M) {
                self.show_chat = !self.show_chat;
                if self.show_chat {
                    self.show_roster = false;
                    self.show_polls = false;
                    self.unread_chat_count = 0;
                }
            }
        }

        // 3. Escape key (Dismiss modals/drawers)
        if input.key_pressed(egui::Key::Escape) {
            if self.show_leave_confirmation {
                self.show_leave_confirmation = false;
            } else if self.kick_confirmation_target.is_some() {
                self.kick_confirmation_target = None;
            } else if self.show_chat || self.show_roster || self.show_polls {
                self.show_chat = false;
                self.show_roster = false;
                self.show_polls = false;
            } else if self.is_whiteboard_active {
                self.is_whiteboard_active = false;
            }
        }
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
                    self.roster = roster
                        .into_iter()
                        .filter(|p| Some(p.participant_id) != self.my_participant_id)
                        .collect();
                }
                ServerMessage::ParticipantJoined { participant } => {
                    if Some(participant.participant_id) != self.my_participant_id {
                        self.roster
                            .retain(|p| p.participant_id != participant.participant_id);
                        self.roster.push(participant);
                    }
                }
                ServerMessage::ParticipantLeft { participant_id, .. } => {
                    self.roster.retain(|p| p.participant_id != participant_id);
                    self.active_speaker_ids.remove(&participant_id);
                }
                ServerMessage::ParticipantMuteChanged {
                    participant_id,
                    kind,
                    muted,
                } => {
                    if let Some(p) = self
                        .roster
                        .iter_mut()
                        .find(|p| p.participant_id == participant_id)
                    {
                        if kind == "audio" {
                            p.is_audio_muted = muted;
                        }
                        if kind == "video" {
                            p.is_video_muted = muted;
                        }
                        if kind == "screen_share" {
                            p.is_screen_sharing = !muted;
                        }
                    }
                }
                ServerMessage::ActiveSpeakers { ranked } => {
                    self.active_speaker_ids =
                        ranked.into_iter().map(|spk| spk.participant_id).collect();
                }
                ServerMessage::Chat {
                    id,
                    from_id,
                    from_name,
                    body,
                    sent_at,
                } => {
                    // sent_at is a server-provided RFC3339 timestamp: parse it
                    // safely instead of slicing bytes (which could panic on
                    // short strings or non-UTF-8 boundaries).
                    let sent_at_hhmm = chrono::DateTime::parse_from_rfc3339(&sent_at)
                        .map(|dt| dt.format("%H:%M").to_string())
                        .unwrap_or_else(|_| {
                            sent_at.get(11..16).unwrap_or(sent_at.as_str()).to_string()
                        });
                    self.chat_messages.push(ChatMessageDto {
                        id,
                        from_id,
                        from_name,
                        body,
                        sent_at: sent_at_hhmm, // HH:MM
                    });
                    if !self.show_chat {
                        self.unread_chat_count += 1;
                    }
                }
                ServerMessage::Reaction { emoji, .. } => {
                    self.spawn_reaction(emoji);
                }
                ServerMessage::CaptionBroadcast {
                    participant_id,
                    speaker_name,
                    text,
                    is_final,
                    language,
                    timestamp_ms,
                } => {
                    self.add_caption_chunk(CaptionChunkDto {
                        participant_id,
                        speaker_name,
                        text,
                        is_final,
                        language,
                        timestamp_ms,
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
                ServerMessage::WaitingRoomStatus {
                    is_waiting,
                    message,
                } => {
                    self.is_waiting_in_lobby = is_waiting;
                    self.waiting_room_message = message;
                }
                ServerMessage::ParticipantWaiting { participant } => {
                    if !self
                        .waiting_participants
                        .iter()
                        .any(|p| p.participant_id == participant.participant_id)
                    {
                        self.waiting_participants.push(participant);
                    }
                }
                ServerMessage::ParticipantAdmitted { participant_id } => {
                    if Some(participant_id) == self.my_participant_id {
                        self.is_waiting_in_lobby = false;
                        self.waiting_room_message = None;
                    }
                    self.waiting_participants
                        .retain(|p| p.participant_id != participant_id);
                }
                ServerMessage::ParticipantRejected { participant_id } => {
                    if Some(participant_id) == self.my_participant_id {
                        self.leave_meeting();
                        self.error_message =
                            Some("The host rejected your request to join the meeting.".to_string());
                    }
                    self.waiting_participants
                        .retain(|p| p.participant_id != participant_id);
                }
                ServerMessage::MeetingPolicyChanged { policy } => {
                    self.is_room_locked = policy.is_locked;
                    self.is_watermark_enabled = policy.watermark_enabled;
                    self.meeting_policy = policy;
                }
                ServerMessage::MeetingEnded { .. } => {
                    self.leave_meeting();
                }
                ServerMessage::Pong { seq } => {
                    if let Some(rtt) = self.client.take_rtt_ms(seq) {
                        self.rtt_ms = rtt;
                    }
                }
                ServerMessage::Error { code, message } => {
                    tracing::warn!("Server error {code}: {message}");
                    self.error_message = Some(format!("{code}: {message}"));
                }
                ServerMessage::PublishOk { sdp } => {
                    if let Some(engine) = self.rtc_engine.clone() {
                        self.tokio_rt.spawn(async move {
                            let engine = engine.lock().await;
                            if let Err(e) = engine.apply_publish_answer(sdp).await {
                                tracing::warn!("failed to apply publish answer: {e}");
                            }
                        });
                    }
                }
                ServerMessage::SubscribeOffer { sdp, mapping } => {
                    if let Some(engine) = self.rtc_engine.clone() {
                        if let Some(out_tx) = self.client.outgoing_tx.clone() {
                            self.tokio_rt.spawn(async move {
                                let engine = engine.lock().await;
                                match engine.apply_subscribe_offer(sdp, mapping).await {
                                    Ok(answer_sdp) => {
                                        let msg =
                                            ClientMessage::SubscribeAnswer { sdp: answer_sdp };
                                        if let Err(e) = out_tx.try_send(msg) {
                                            tracing::warn!(
                                                "failed to enqueue subscribe answer: {e}"
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("failed to apply subscribe offer: {e}")
                                    }
                                }
                            });
                        }
                    }
                }
                ServerMessage::IceCandidate {
                    target,
                    candidate,
                    sdp_mid,
                    sdp_mline_index,
                    username_fragment,
                } => match CandidateTarget::try_from(target.as_str()) {
                    Ok(target) => {
                        if let Some(engine) = self.rtc_engine.clone() {
                            self.tokio_rt.spawn(async move {
                                let engine = engine.lock().await;
                                if let Err(e) = engine
                                    .add_ice_candidate(
                                        target,
                                        candidate,
                                        sdp_mid,
                                        sdp_mline_index,
                                        username_fragment,
                                    )
                                    .await
                                {
                                    tracing::warn!("failed to add remote ICE candidate: {e}");
                                }
                            });
                        }
                    }
                    Err(e) => {
                        tracing::warn!("ignoring ICE candidate with invalid target: {e}");
                    }
                },
                unhandled => {
                    tracing::debug!("Ignoring unimplemented server message: {unhandled:?}");
                }
            }
        }
    }
}

impl eframe::App for ConferApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Dark theme matching Confer design system (Obsidian & Deep Zinc)
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
        let mut visuals = ctx.style().visuals.clone();
        visuals.panel_fill = Theme::CANVAS;
        visuals.window_fill = Theme::SURFACE_1;
        ctx.set_visuals(visuals);

        self.poll_connect_result();
        self.poll_incoming_messages();
        self.handle_global_shortcuts(ctx);

        // Drain remote decoded video frames (Step 5)
        if let Some(rx) = &mut self.remote_frame_rx {
            while let Ok((publisher_id, image)) = rx.try_recv() {
                let tex = ctx.load_texture(
                    format!("remote_video_{}", publisher_id),
                    image,
                    egui::TextureOptions::LINEAR,
                );
                self.remote_video_textures.insert(publisher_id, tex);
            }
        }

        // Update local live camera frame texture and publish to WebRTC track (Step 4)
        if !self.is_camera_off {
            self.camera_capturer.set_filter(self.active_filter);
            self.camera_capturer
                .set_background(self.virtual_bg_mode.clone());
            if let Some((frame_id, frame)) = self
                .camera_capturer
                .get_latest_frame_if_newer(self.last_rendered_frame_id)
            {
                // If not sharing screen, publish local camera frame to WebRTC track
                if !self.is_screen_sharing {
                    if let Some(tx) = &self.local_frame_tx {
                        // The hardware camera may return a resolution other than the
                        // encoder's fixed 320x180 (e.g. "closest match" webcam modes),
                        // so always resample to the exact encoder dimensions rather
                        // than assuming `frame` is already 320x180.
                        let img_w = frame.width();
                        let img_h = frame.height();
                        let mut rgb = Vec::with_capacity(320 * 180 * 3);
                        for y in 0..180 {
                            let src_y = (y * img_h) / 180;
                            for x in 0..320 {
                                let src_x = (x * img_w) / 320;
                                let p = frame.pixels[src_y * img_w + src_x];
                                rgb.push(p.r());
                                rgb.push(p.g());
                                rgb.push(p.b());
                            }
                        }
                        let _ = tx.try_send(rgb);
                    }
                }

                self.local_video_texture = Some(ctx.load_texture(
                    "local_camera_feed",
                    frame,
                    egui::TextureOptions::LINEAR,
                ));
                self.last_rendered_frame_id = frame_id;
            }
        } else {
            self.local_video_texture = None;
            self.last_rendered_frame_id = 0;
        }

        // Update real-time screen share texture and publish to WebRTC track (Step 4)
        if self.is_screen_sharing {
            if let Some(err) = self.screen_capturer.take_error() {
                match err {
                    crate::media::CaptureError::PortalCancelled => {
                        tracing::info!("User cancelled screen share picker dialog");
                    }
                    other => {
                        tracing::warn!("Screen capture failed: {other}");
                        self.error_message = Some(format!("Screen share error: {other}"));
                    }
                }
                self.is_screen_sharing = false;
                self.screen_share_texture = None;
                self.last_screen_frame_id = 0;
                self.client.send_message(ClientMessage::SetMute {
                    kind: "screen_share".to_string(),
                    muted: true,
                });
            } else if let Some((frame_id, frame)) = self
                .screen_capturer
                .get_latest_frame_if_newer(self.last_screen_frame_id)
            {
                // Publish screen share frame to WebRTC track
                if let Some(tx) = &self.local_frame_tx {
                    let img_w = frame.width();
                    let img_h = frame.height();
                    let mut rgb = Vec::with_capacity(320 * 180 * 3);
                    for y in 0..180 {
                        let src_y = (y * img_h) / 180;
                        for x in 0..320 {
                            let src_x = (x * img_w) / 320;
                            let p = frame.pixels[src_y * img_w + src_x];
                            rgb.push(p.r());
                            rgb.push(p.g());
                            rgb.push(p.b());
                        }
                    }
                    let _ = tx.try_send(rgb);
                }

                self.screen_share_texture = Some(ctx.load_texture(
                    "screen_share_feed",
                    frame,
                    egui::TextureOptions::LINEAR,
                ));
                self.last_screen_frame_id = frame_id;
            }
        } else {
            self.screen_share_texture = None;
            self.last_screen_frame_id = 0;
        }

        // Request continuous repaint for smooth 60fps animations
        ctx.request_repaint_after(Duration::from_millis(16));

        egui::CentralPanel::default().show(ctx, |ui| match self.view_state {
            ViewState::Lobby => lobby::render_lobby(self, ui),
            ViewState::MeetingRoom => {
                if self.is_waiting_in_lobby {
                    waiting_lobby::render_waiting_lobby(self, ui);
                } else {
                    meeting_room::render_meeting_room(self, ui);
                }
            }
        });
    }
}

/// Pseudo-random float in [0, 1) for cosmetic reaction placement.
/// The `rand` crate is not a dependency, so this hashes a monotonically
/// increasing atomic counter (splitmix64-style) — no clock reads, no unwrap.
fn rand_float() -> f32 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);

    let mut x = COUNTER.fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    (x % 1000) as f32 / 1000.0
}
