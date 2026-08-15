// Confer Web Client - Real-Time Collaboration Engine (ES6)
(() => {
  'use strict';

  // --- State ---
  const state = {
    lang: 'en',
    user: {
      id: '11111111-1111-1111-1111-111111111111',
      name: 'Host User (Dev)',
      email: 'host@confer.local',
      role: 'host'
    },
    meeting: null,
    roomToken: null,
    ws: null,
    pingInterval: null,
    localStream: null,
    screenStream: null,
    audioContext: null,
    analyser: null,
    isMicMuted: false,
    isCamOff: false,
    isScreenSharing: false,
    isHandRaised: false,
    isDenoiseActive: true,
    isWhiteboardActive: false,
    isWaitingRoom: false,
    isWatermarkEnabled: false,
    meetingPolicy: {
      is_locked: false,
      waiting_room_enabled: false,
      allow_screen_share: true,
      allow_chat: true,
      allow_unmute: true,
      watermark_enabled: false
    },
    roster: [],
    waitingParticipants: [],
    chatMessages: [],
    unreadChat: 0,
    polls: [],
    whiteboardStrokes: [],
    rtt: 24,
    lastPingSeq: 0,
    pingStart: 0
  };

  // --- i18n Strings ---
  const i18n = {
    en: {
      lobbyTitle: "Ready to join?",
      lobbySubtitle: "Verify your audio and camera before entering the room.",
      labelName: "Display Name",
      createTitle: "Create New Meeting",
      btnCreate: "Start Meeting",
      joinTitle: "Join Existing Meeting",
      btnJoin: "Join Room",
      waitingTitle: "Please wait, the host will let you in soon",
      mute: "Mute",
      unmute: "Unmute",
      camera: "Camera",
      share: "Share",
      whiteboard: "Whiteboard",
      hand: "Hand",
      react: "React",
      chat: "Chat",
      people: "People",
      polls: "Polls",
      security: "Security",
      leave: "Leave"
    },
    es: {
      lobbyTitle: "¿Listo para unirte?",
      lobbySubtitle: "Verifica tu audio y cámara antes de entrar a la sala.",
      labelName: "Nombre visible",
      createTitle: "Crear Nueva Reunión",
      btnCreate: "Iniciar Reunión",
      joinTitle: "Unirse a Reunión Existente",
      btnJoin: "Entrar a Sala",
      waitingTitle: "Por favor espera, el anfitrión te admitirá pronto",
      mute: "Silenciar",
      unmute: "Reactivar",
      camera: "Cámara",
      share: "Compartir",
      whiteboard: "Pizarra",
      hand: "Mano",
      react: "Reaccionar",
      chat: "Chat",
      people: "Participantes",
      polls: "Encuestas",
      security: "Seguridad",
      leave: "Salir"
    }
  };

  // --- DOM Elements ---
  const els = {
    headerMeetingInfo: document.getElementById('header-meeting-info'),
    roomTitleDisplay: document.getElementById('room-title-display'),
    roomCodeDisplay: document.getElementById('room-code-display'),
    recordingBadge: document.getElementById('recording-badge'),
    streamBadge: document.getElementById('stream-badge'),
    rttMetric: document.getElementById('rtt-metric'),
    userAvatar: document.getElementById('user-avatar'),
    userDisplayName: document.getElementById('user-display-name'),
    langToggleBtn: document.getElementById('lang-toggle-btn'),
    diagnosticsBtn: document.getElementById('diagnostics-btn'),

    // Views
    lobbyView: document.getElementById('lobby-view'),
    waitingRoomView: document.getElementById('waiting-room-view'),
    meetingView: document.getElementById('meeting-view'),

    // Lobby
    localPreviewVideo: document.getElementById('local-preview-video'),
    previewAvatarPlaceholder: document.getElementById('preview-avatar-placeholder'),
    previewAvatarLetter: document.getElementById('preview-avatar-letter'),
    previewAudioMeter: document.getElementById('preview-audio-meter'),
    previewMicBtn: document.getElementById('preview-mic-btn'),
    previewCamBtn: document.getElementById('preview-cam-btn'),
    previewDenoiseBtn: document.getElementById('preview-denoise-btn'),
    displayNameInput: document.getElementById('display-name-input'),
    createTitleInput: document.getElementById('create-title-input'),
    createMeetingBtn: document.getElementById('create-meeting-btn'),
    joinCodeInput: document.getElementById('join-code-input'),
    joinMeetingBtn: document.getElementById('join-meeting-btn'),
    waitingRoomTitle: document.getElementById('waiting-room-title'),
    leaveWaitingBtn: document.getElementById('leave-waiting-btn'),

    // Meeting Stage
    videoGrid: document.getElementById('video-grid'),
    whiteboardContainer: document.getElementById('whiteboard-container'),
    whiteboardCanvas: document.getElementById('whiteboard-canvas'),
    wbColorPicker: document.getElementById('wb-color-picker'),
    wbStrokeWidth: document.getElementById('wb-stroke-width'),
    wbUndoBtn: document.getElementById('wb-undo-btn'),
    wbClearBtn: document.getElementById('wb-clear-btn'),
    wbCloseBtn: document.getElementById('wb-close-btn'),
    watermarkOverlay: document.getElementById('watermark-overlay'),
    watermarkPattern: document.getElementById('watermark-pattern'),
    reactionsContainer: document.getElementById('reactions-container'),

    // Side Drawer
    sideDrawer: document.getElementById('side-drawer'),
    drawerTitle: document.getElementById('drawer-title'),
    drawerCloseBtn: document.getElementById('drawer-close-btn'),
    drawerChat: document.getElementById('drawer-chat'),
    drawerRoster: document.getElementById('drawer-roster'),
    drawerPolls: document.getElementById('drawer-polls'),
    drawerSecurity: document.getElementById('drawer-security'),
    chatMessagesContainer: document.getElementById('chat-messages-container'),
    chatInputForm: document.getElementById('chat-input-form'),
    chatMessageInput: document.getElementById('chat-message-input'),
    chatUnreadBadge: document.getElementById('chat-unread-badge'),

    // Roster
    rosterHostTabs: document.getElementById('roster-host-tabs'),
    tabInMeeting: document.getElementById('tab-in-meeting'),
    tabWaitingRoom: document.getElementById('tab-waiting-room'),
    inMeetingCount: document.getElementById('in-meeting-count'),
    waitingCount: document.getElementById('waiting-count'),
    rosterInMeetingList: document.getElementById('roster-in-meeting-list'),
    rosterWaitingList: document.getElementById('roster-waiting-list'),
    admitAllBtn: document.getElementById('admit-all-btn'),
    waitingParticipantsContainer: document.getElementById('waiting-participants-container'),
    dockRosterCount: document.getElementById('dock-roster-count'),

    // Polls
    pollQuestionInput: document.getElementById('poll-question-input'),
    pollOptionsInputs: document.getElementById('poll-options-inputs'),
    pollAddOptBtn: document.getElementById('poll-add-opt-btn'),
    pollMultichoiceCheck: document.getElementById('poll-multichoice-check'),
    pollAnonCheck: document.getElementById('poll-anon-check'),
    pollLaunchBtn: document.getElementById('poll-launch-btn'),
    activePollsContainer: document.getElementById('active-polls-container'),

    // Security Policies
    policyLockCheck: document.getElementById('policy-lock-check'),
    policyWaitingCheck: document.getElementById('policy-waiting-check'),
    policyScreenshareCheck: document.getElementById('policy-screenshare-check'),
    policyChatCheck: document.getElementById('policy-chat-check'),
    policyUnmuteCheck: document.getElementById('policy-unmute-check'),
    policyWatermarkCheck: document.getElementById('policy-watermark-check'),

    // Bottom Dock
    dockMicBtn: document.getElementById('dock-mic-btn'),
    dockCamBtn: document.getElementById('dock-cam-btn'),
    dockScreenshareBtn: document.getElementById('dock-screenshare-btn'),
    dockWhiteboardBtn: document.getElementById('dock-whiteboard-btn'),
    dockHandBtn: document.getElementById('dock-hand-btn'),
    dockReactionsBtn: document.getElementById('dock-reactions-btn'),
    dockChatBtn: document.getElementById('dock-chat-btn'),
    dockRosterBtn: document.getElementById('dock-roster-btn'),
    dockPollsBtn: document.getElementById('dock-polls-btn'),
    dockSecurityBtn: document.getElementById('dock-security-btn'),
    dockLeaveBtn: document.getElementById('dock-leave-btn'),

    // Diagnostics Modal
    diagnosticsModal: document.getElementById('diagnostics-modal'),
    diagCloseBtn: document.getElementById('diag-close-btn'),
    hudRtt: document.getElementById('hud-rtt'),
    hudLoss: document.getElementById('hud-loss')
  };

  // --- Initialize App ---
  async function init() {
    setupEventListeners();
    setupWhiteboard();
    setupKeyboardShortcuts();
    await initMediaPreview();
  }

  // --- Media & Audio Visualizer ---
  async function initMediaPreview() {
    try {
      state.localStream = await navigator.mediaDevices.getUserMedia({
        video: { width: { ideal: 1280 }, height: { ideal: 720 } },
        audio: true
      });
      els.localPreviewVideo.srcObject = state.localStream;
      els.previewAvatarPlaceholder.style.display = 'none';

      // Setup Web Audio Analyser for mic meter
      state.audioContext = new (window.AudioContext || window.webkitAudioContext)();
      const source = state.audioContext.createMediaStreamSource(state.localStream);
      state.analyser = state.audioContext.createAnalyser();
      state.analyser.fftSize = 64;
      source.connect(state.analyser);

      const buffer = new Uint8Array(state.analyser.frequencyBinCount);
      function updateMeter() {
        if (state.analyser && !state.isMicMuted) {
          state.analyser.getByteFrequencyData(buffer);
          let sum = 0;
          for (let i = 0; i < buffer.length; i++) sum += buffer[i];
          const avg = sum / buffer.length;
          const pct = Math.min(100, Math.round((avg / 128) * 100));
          els.previewAudioMeter.style.width = `${pct}%`;
        } else {
          els.previewAudioMeter.style.width = '0%';
        }
        requestAnimationFrame(updateMeter);
      }
      requestAnimationFrame(updateMeter);
    } catch (e) {
      console.warn("Media devices access warning:", e);
      els.previewAvatarPlaceholder.style.display = 'flex';
    }
  }

  // --- Event Listeners ---
  function setupEventListeners() {
    // Language Toggle
    els.langToggleBtn.addEventListener('click', () => {
      state.lang = state.lang === 'en' ? 'es' : 'en';
      updateI18n();
    });

    // Device Toggles (Lobby & Dock)
    els.previewMicBtn.addEventListener('click', toggleMic);
    els.dockMicBtn.addEventListener('click', toggleMic);
    els.previewCamBtn.addEventListener('click', toggleCam);
    els.dockCamBtn.addEventListener('click', toggleCam);

    els.previewDenoiseBtn.addEventListener('click', () => {
      state.isDenoiseActive = !state.isDenoiseActive;
      els.previewDenoiseBtn.classList.toggle('active', state.isDenoiseActive);
      els.previewDenoiseBtn.textContent = state.isDenoiseActive ? '⚡ Denoise' : '⚡ Denoise Off';
    });

    // Create & Join Actions
    els.createMeetingBtn.addEventListener('click', createMeeting);
    els.joinMeetingBtn.addEventListener('click', joinMeetingByCode);
    els.leaveWaitingBtn.addEventListener('click', leaveMeeting);
    els.dockLeaveBtn.addEventListener('click', leaveMeeting);

    // Screen Share
    els.dockScreenshareBtn.addEventListener('click', toggleScreenShare);

    // Whiteboard Toggle
    els.dockWhiteboardBtn.addEventListener('click', toggleWhiteboard);
    els.wbCloseBtn.addEventListener('click', () => toggleWhiteboard(false));

    // Hand Raise
    els.dockHandBtn.addEventListener('click', toggleHandRaise);

    // Reactions Menu
    document.querySelectorAll('.emoji-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        sendReaction(e.target.dataset.emoji);
      });
    });

    // Drawer Toggles
    els.dockChatBtn.addEventListener('click', () => toggleDrawer('chat'));
    els.dockRosterBtn.addEventListener('click', () => toggleDrawer('roster'));
    els.dockPollsBtn.addEventListener('click', () => toggleDrawer('polls'));
    els.dockSecurityBtn.addEventListener('click', () => toggleDrawer('security'));
    els.drawerCloseBtn.addEventListener('click', () => toggleDrawer(null));

    // Roster Subtabs
    els.tabInMeeting.addEventListener('click', () => {
      els.tabInMeeting.classList.add('active');
      els.tabWaitingRoom.classList.remove('active');
      els.rosterInMeetingList.style.display = 'flex';
      els.rosterWaitingList.style.display = 'none';
    });

    els.tabWaitingRoom.addEventListener('click', () => {
      els.tabWaitingRoom.classList.add('active');
      els.tabInMeeting.classList.remove('active');
      els.rosterInMeetingList.style.display = 'none';
      els.rosterWaitingList.style.display = 'flex';
    });

    els.admitAllBtn.addEventListener('click', () => {
      sendWsMessage({ type: 'admit_all' });
      state.waitingParticipants = [];
      updateWaitingRosterUI();
    });

    // Chat Send Form
    els.chatInputForm.addEventListener('submit', (e) => {
      e.preventDefault();
      const text = els.chatMessageInput.value.trim();
      if (!text) return;
      sendWsMessage({
        type: 'chat_message',
        message: text
      });
      els.chatMessageInput.value = '';
    });

    // Polls Creator
    els.pollAddOptBtn.addEventListener('click', () => {
      const count = els.pollOptionsInputs.querySelectorAll('.poll-opt-input').length;
      if (count >= 6) return;
      const inp = document.createElement('input');
      inp.type = 'text';
      inp.className = 'poll-opt-input';
      inp.placeholder = `Option ${count + 1}`;
      els.pollOptionsInputs.appendChild(inp);
    });

    els.pollLaunchBtn.addEventListener('click', createPoll);

    // Host Security Policies
    [els.policyLockCheck, els.policyWaitingCheck, els.policyScreenshareCheck, els.policyChatCheck, els.policyUnmuteCheck, els.policyWatermarkCheck].forEach(chk => {
      chk.addEventListener('change', updateHostPolicies);
    });

    // Diagnostics Modal
    els.diagnosticsBtn.addEventListener('click', () => {
      els.diagnosticsModal.style.display = 'flex';
    });
    els.diagCloseBtn.addEventListener('click', () => {
      els.diagnosticsModal.style.display = 'none';
    });
    els.diagnosticsModal.addEventListener('click', (e) => {
      if (e.target === els.diagnosticsModal) els.diagnosticsModal.style.display = 'none';
    });

    // Copy Join Code
    els.roomCodeDisplay.addEventListener('click', () => {
      if (state.meeting?.join_code) {
        navigator.clipboard.writeText(state.meeting.join_code);
        const orig = els.roomCodeDisplay.textContent;
        els.roomCodeDisplay.textContent = 'COPIED!';
        setTimeout(() => els.roomCodeDisplay.textContent = orig, 1500);
      }
    });
  }

  // --- Push to Talk & Keyboard Shortcuts ---
  function setupKeyboardShortcuts() {
    let pttActive = false;

    window.addEventListener('keydown', (e) => {
      // Spacebar for Push-to-Talk (only if not typing in text input)
      if (e.code === 'Space' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
        if (!pttActive && state.isMicMuted) {
          pttActive = true;
          toggleMic(false); // temporary unmute
        }
      }

      // Ctrl+M or Cmd+M: Toggle mic
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'm') {
        e.preventDefault();
        toggleMic();
      }

      // Ctrl+E or Cmd+E: Toggle Camera
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'e') {
        e.preventDefault();
        toggleCam();
      }

      // Ctrl+S or Cmd+S: Toggle Screen Share
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        toggleScreenShare();
      }
    });

    window.addEventListener('keyup', (e) => {
      if (e.code === 'Space' && pttActive) {
        pttActive = false;
        toggleMic(true); // mute again
      }
    });
  }

  // --- Media Controls ---
  function toggleMic(forceMute = null) {
    state.isMicMuted = forceMute !== null ? forceMute : !state.isMicMuted;
    if (state.localStream) {
      state.localStream.getAudioTracks().forEach(t => t.enabled = !state.isMicMuted);
    }
    const label = state.isMicMuted ? i18n[state.lang].unmute : i18n[state.lang].mute;
    els.dockMicBtn.querySelector('span').textContent = label;
    els.dockMicBtn.classList.toggle('active', !state.isMicMuted);
    els.previewMicBtn.classList.toggle('active', !state.isMicMuted);
    els.previewMicBtn.classList.toggle('muted', state.isMicMuted);
    sendWsMessage({ type: 'mute_audio', is_muted: state.isMicMuted });
  }

  function toggleCam() {
    state.isCamOff = !state.isCamOff;
    if (state.localStream) {
      state.localStream.getVideoTracks().forEach(t => t.enabled = !state.isCamOff);
    }
    els.dockCamBtn.classList.toggle('active', !state.isCamOff);
    els.previewCamBtn.classList.toggle('active', !state.isCamOff);
    els.previewAvatarPlaceholder.style.display = state.isCamOff ? 'flex' : 'none';
    sendWsMessage({ type: 'mute_video', is_muted: state.isCamOff });
  }

  async function toggleScreenShare() {
    if (state.isScreenSharing) {
      if (state.screenStream) {
        state.screenStream.getTracks().forEach(t => t.stop());
      }
      state.isScreenSharing = false;
      els.dockScreenshareBtn.classList.remove('active');
      sendWsMessage({ type: 'screen_share', is_sharing: false });
      renderVideoGrid();
    } else {
      try {
        state.screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: true });
        state.isScreenSharing = true;
        els.dockScreenshareBtn.classList.add('active');
        state.screenStream.getVideoTracks()[0].onended = () => toggleScreenShare();
        sendWsMessage({ type: 'screen_share', is_sharing: true });
        renderVideoGrid();
      } catch (e) {
        console.warn("Screen share cancelled or denied:", e);
      }
    }
  }

  function toggleWhiteboard(forceState = null) {
    state.isWhiteboardActive = forceState !== null ? forceState : !state.isWhiteboardActive;
    els.whiteboardContainer.style.display = state.isWhiteboardActive ? 'flex' : 'none';
    els.dockWhiteboardBtn.classList.toggle('active', state.isWhiteboardActive);
    if (state.isWhiteboardActive) resizeWhiteboard();
  }

  function toggleHandRaise() {
    state.isHandRaised = !state.isHandRaised;
    els.dockHandBtn.classList.toggle('active', state.isHandRaised);
    sendWsMessage({ type: 'hand_raise', is_raised: state.isHandRaised });
  }

  function sendReaction(emoji) {
    sendWsMessage({ type: 'reaction', emoji });
    triggerEmojiParticle(emoji);
  }

  function triggerEmojiParticle(emoji) {
    const el = document.createElement('div');
    el.className = 'reaction-particle';
    el.textContent = emoji;
    el.style.left = `${Math.random() * 60 + 20}px`;
    els.reactionsContainer.appendChild(el);
    setTimeout(() => el.remove(), 2500);
  }

  // --- REST & Signaling Handshake ---
  async function createMeeting() {
    const displayName = els.displayNameInput.value.trim() || 'Host User';
    const title = els.createTitleInput.value.trim() || 'Sprint Planning';
    state.user.name = displayName;

    try {
      // 1. Dev login as host
      const loginRes = await fetch('/api/auth/dev-login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: state.user.email, role: 'host' })
      });
      const loginData = await loginRes.json();

      // 2. Create meeting
      const createRes = await fetch('/api/meetings', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${loginData.token}`
        },
        body: JSON.stringify({ title, max_participants: 50 })
      });
      const meetingData = await createRes.json();

      // 3. Join meeting as host
      const joinRes = await fetch(`/api/meetings/${meetingData.id}/join`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${loginData.token}`
        },
        body: JSON.stringify({ display_name: displayName, client_info: 'Confer Web 1.0' })
      });
      const joinData = await joinRes.json();

      state.meeting = meetingData;
      state.roomToken = joinData.room_token;
      state.user.role = 'host';

      connectWebSocket(joinData.room_token);
      switchToMeetingView();
    } catch (err) {
      alert(`Failed to start meeting: ${err.message}`);
    }
  }

  async function joinMeetingByCode() {
    const displayName = els.displayNameInput.value.trim() || 'Participant';
    const code = els.joinCodeInput.value.trim();
    if (!code) return alert("Please enter a valid join code.");

    try {
      // 1. Get meeting by code
      const getRes = await fetch(`/api/meetings/code/${code}`);
      if (!getRes.ok) throw new Error("Meeting not found with this code");
      const meetingData = await getRes.json();

      // 2. Dev login as participant
      const loginRes = await fetch('/api/auth/dev-login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: `guest_${Date.now()}@confer.local`, role: 'participant' })
      });
      const loginData = await loginRes.json();

      // 3. Join meeting
      const joinRes = await fetch(`/api/meetings/${meetingData.id}/join`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${loginData.token}`
        },
        body: JSON.stringify({ display_name: displayName, client_info: 'Confer Web 1.0' })
      });
      const joinData = await joinRes.json();

      state.meeting = meetingData;
      state.roomToken = joinData.room_token;
      state.user.role = 'participant';

      if (joinData.is_waiting) {
        state.isWaitingRoom = true;
        switchToWaitingView();
      } else {
        switchToMeetingView();
      }

      connectWebSocket(joinData.room_token);
    } catch (err) {
      alert(`Join error: ${err.message}`);
    }
  }

  function connectWebSocket(token) {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${location.host}/v1/signal?token=${encodeURIComponent(token)}`;

    state.ws = new WebSocket(wsUrl, 'confer.v1');

    state.ws.onopen = () => {
      startPingLoop();
    };

    state.ws.onmessage = (evt) => {
      try {
        const msg = JSON.parse(evt.data);
        handleServerMessage(msg);
      } catch (e) {
        console.warn("WS parse error:", e);
      }
    };

    state.ws.onclose = () => {
      clearInterval(state.pingInterval);
    };
  }

  function startPingLoop() {
    clearInterval(state.pingInterval);
    state.pingInterval = setInterval(() => {
      state.lastPingSeq++;
      state.pingStart = performance.now();
      sendWsMessage({ type: 'ping', seq: state.lastPingSeq });
    }, 5000);
  }

  function sendWsMessage(obj) {
    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
      state.ws.send(JSON.stringify(obj));
    }
  }

  // --- WebSocket Protocol Handler ---
  function handleServerMessage(msg) {
    switch (msg.type) {
      case 'pong':
        state.rtt = Math.max(1, Math.round(performance.now() - state.pingStart));
        els.rttMetric.textContent = `${state.rtt} ms`;
        els.hudRtt.textContent = `${state.rtt} ms`;
        break;

      case 'roster_sync':
        state.roster = msg.participants || [];
        updateRosterUI();
        renderVideoGrid();
        break;

      case 'participant_joined':
        if (!state.roster.some(p => p.id === msg.participant.id)) {
          state.roster.push(msg.participant);
          updateRosterUI();
          renderVideoGrid();
        }
        break;

      case 'participant_left':
        state.roster = state.roster.filter(p => p.id !== msg.participant_id);
        updateRosterUI();
        renderVideoGrid();
        break;

      case 'chat_message':
        state.chatMessages.push(msg.message);
        appendChatMessage(msg.message);
        if (els.sideDrawer.style.display === 'none' || !els.drawerChat.classList.contains('active')) {
          state.unreadChat++;
          els.chatUnreadBadge.textContent = state.unreadChat;
          els.chatUnreadBadge.style.display = 'inline-block';
        }
        break;

      case 'reaction':
        triggerEmojiParticle(msg.emoji);
        break;

      case 'active_speaker':
        highlightActiveSpeaker(msg.participant_id);
        break;

      case 'waiting_room_status':
        if (msg.is_waiting) switchToWaitingView();
        break;

      case 'participant_waiting':
        if (!state.waitingParticipants.some(p => p.participant_id === msg.participant.participant_id)) {
          state.waitingParticipants.push(msg.participant);
          updateWaitingRosterUI();
        }
        break;

      case 'participant_admitted':
        if (state.isWaitingRoom) {
          state.isWaitingRoom = false;
          switchToMeetingView();
        }
        state.waitingParticipants = state.waitingParticipants.filter(p => p.participant_id !== msg.participant_id);
        updateWaitingRosterUI();
        break;

      case 'participant_rejected':
        if (state.isWaitingRoom) {
          alert("The host rejected your join request.");
          leaveMeeting();
        }
        state.waitingParticipants = state.waitingParticipants.filter(p => p.participant_id !== msg.participant_id);
        updateWaitingRosterUI();
        break;

      case 'policy_changed':
        state.meetingPolicy = msg.policy;
        state.isWatermarkEnabled = msg.is_watermark_enabled;
        applyPolicies();
        break;

      case 'whiteboard_stroke':
        state.whiteboardStrokes.push(msg.stroke);
        drawStrokeOnCanvas(msg.stroke);
        break;

      case 'whiteboard_clear':
        clearLocalWhiteboard();
        break;

      case 'poll_created':
        if (!state.polls.some(p => p.id === msg.poll.id)) {
          state.polls.push(msg.poll);
          renderPolls();
        }
        break;

      case 'poll_updated':
        const idx = state.polls.findIndex(p => p.id === msg.poll.id);
        if (idx >= 0) state.polls[idx] = msg.poll;
        else state.polls.push(msg.poll);
        renderPolls();
        break;

      case 'live_stream_status':
        els.streamBadge.style.display = msg.is_streaming ? 'inline-block' : 'none';
        break;
    }
  }

  // --- View Transitions ---
  function switchToMeetingView() {
    els.lobbyView.classList.remove('active');
    els.lobbyView.style.display = 'none';
    els.waitingRoomView.style.display = 'none';
    els.meetingView.style.display = 'flex';

    els.headerMeetingInfo.style.display = 'flex';
    els.roomTitleDisplay.textContent = state.meeting?.title || 'Meeting Room';
    els.roomCodeDisplay.textContent = `CODE: ${state.meeting?.join_code || '------'}`;
    els.dockSecurityBtn.style.display = state.user.role === 'host' ? 'flex' : 'none';
    els.rosterHostTabs.style.display = state.user.role === 'host' ? 'flex' : 'none';

    renderVideoGrid();
  }

  function switchToWaitingView() {
    els.lobbyView.classList.remove('active');
    els.lobbyView.style.display = 'none';
    els.meetingView.style.display = 'none';
    els.waitingRoomView.style.display = 'flex';
    els.waitingRoomTitle.textContent = `Meeting: ${state.meeting?.title || 'In Session'}`;
  }

  function leaveMeeting() {
    if (state.ws) {
      state.ws.close();
      state.ws = null;
    }
    clearInterval(state.pingInterval);
    state.roster = [];
    state.chatMessages = [];
    state.polls = [];
    state.waitingParticipants = [];

    els.meetingView.style.display = 'none';
    els.waitingRoomView.style.display = 'none';
    els.lobbyView.style.display = 'flex';
    els.lobbyView.classList.add('active');
    els.headerMeetingInfo.style.display = 'none';
  }

  // --- Video Grid Rendering ---
  function renderVideoGrid() {
    els.videoGrid.innerHTML = '';

    // 1. Local video tile
    const localTile = createVideoTile('You (Local)', true, state.localStream);
    els.videoGrid.appendChild(localTile);

    // 2. Screen share tile if active
    if (state.isScreenSharing && state.screenStream) {
      const screenTile = createVideoTile('Your Screen Share', false, state.screenStream);
      els.videoGrid.appendChild(screenTile);
    }

    // 3. Remote participant tiles
    state.roster.forEach(p => {
      const tile = createVideoTile(p.display_name, false, null, p.id);
      els.videoGrid.appendChild(tile);
    });

    updateWatermarkPattern();
  }

  function createVideoTile(name, isLocal, stream = null, participantId = null) {
    const tile = document.createElement('div');
    tile.className = 'video-tile' + (isLocal ? ' mirrored' : '');
    if (participantId) tile.dataset.participantId = participantId;

    if (stream) {
      const video = document.createElement('video');
      video.autoplay = true;
      video.playsInline = true;
      if (isLocal) video.muted = true;
      video.srcObject = stream;
      tile.appendChild(video);
    } else {
      const placeholder = document.createElement('div');
      placeholder.className = 'video-placeholder';
      const letter = document.createElement('div');
      letter.className = 'avatar-huge';
      letter.textContent = (name || 'U')[0].toUpperCase();
      placeholder.appendChild(letter);
      tile.appendChild(placeholder);
    }

    const overlay = document.createElement('div');
    overlay.className = 'tile-overlay-bottom';
    overlay.innerHTML = `
      <span class="participant-label">${name}</span>
      <div class="tile-indicators"></div>
    `;
    tile.appendChild(overlay);
    return tile;
  }

  function highlightActiveSpeaker(participantId) {
    document.querySelectorAll('.video-tile').forEach(t => t.classList.remove('active-speaker'));
    const tile = document.querySelector(`.video-tile[data-participant-id="${participantId}"]`);
    if (tile) tile.classList.add('active-speaker');
  }

  // --- Drawer Management ---
  function toggleDrawer(tabName) {
    if (!tabName || (els.sideDrawer.style.display === 'flex' && els[`drawer${capitalize(tabName)}`]?.classList.contains('active'))) {
      els.sideDrawer.style.display = 'none';
      return;
    }

    els.sideDrawer.style.display = 'flex';
    document.querySelectorAll('.drawer-tab-pane').forEach(p => p.classList.remove('active'));

    els.drawerTitle.textContent = capitalize(tabName);
    els[`drawer${capitalize(tabName)}`].classList.add('active');

    if (tabName === 'chat') {
      state.unreadChat = 0;
      els.chatUnreadBadge.style.display = 'none';
    }
  }

  function capitalize(s) {
    return s.charAt(0).toUpperCase() + s.slice(1);
  }

  // --- Chat UI ---
  function appendChatMessage(msg) {
    const bubble = document.createElement('div');
    bubble.className = 'chat-bubble';
    const timeStr = new Date(msg.sent_at || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    bubble.innerHTML = `
      <div class="chat-bubble-header">
        <span class="chat-sender">${msg.from_name}</span>
        <span class="chat-time">${timeStr}</span>
      </div>
      <div class="chat-bubble-body">${msg.content}</div>
    `;
    els.chatMessagesContainer.appendChild(bubble);
    els.chatMessagesContainer.scrollTop = els.chatMessagesContainer.scrollHeight;
  }

  // --- Roster & Host Moderation UI ---
  function updateRosterUI() {
    els.inMeetingCount.textContent = state.roster.length + 1;
    els.dockRosterCount.textContent = state.roster.length + 1;
    els.rosterInMeetingList.innerHTML = '';

    // Local user row
    const localRow = document.createElement('div');
    localRow.className = 'roster-row';
    localRow.innerHTML = `
      <div class="roster-user-info">
        <div class="avatar-circle">${state.user.name[0].toUpperCase()}</div>
        <span>${state.user.name} (You)</span>
      </div>
      <span class="badge-tag">${state.user.role}</span>
    `;
    els.rosterInMeetingList.appendChild(localRow);

    // Remote participants
    state.roster.forEach(p => {
      const row = document.createElement('div');
      row.className = 'roster-row';
      row.innerHTML = `
        <div class="roster-user-info">
          <div class="avatar-circle">${p.display_name[0].toUpperCase()}</div>
          <span>${p.display_name}</span>
        </div>
        <div class="roster-actions">
          ${state.user.role === 'host' ? `<button class="btn-xs btn-danger" onclick="window.conferHostKick('${p.id}')">Kick</button>` : ''}
        </div>
      `;
      els.rosterInMeetingList.appendChild(row);
    });
  }

  function updateWaitingRosterUI() {
    els.waitingCount.textContent = state.waitingParticipants.length;
    els.waitingParticipantsContainer.innerHTML = '';

    state.waitingParticipants.forEach(p => {
      const row = document.createElement('div');
      row.className = 'roster-row';
      row.innerHTML = `
        <div class="roster-user-info">
          <div class="avatar-circle">${p.display_name[0].toUpperCase()}</div>
          <div>
            <strong>${p.display_name}</strong>
            <div style="font-size:10px;color:#94a3b8">${p.email || 'Guest'}</div>
          </div>
        </div>
        <div class="roster-actions">
          <button class="btn-xs btn-success" onclick="window.conferHostAdmit('${p.participant_id}')">✓ Admit</button>
          <button class="btn-xs btn-danger" onclick="window.conferHostReject('${p.participant_id}')">✕ Reject</button>
        </div>
      `;
      els.waitingParticipantsContainer.appendChild(row);
    });
  }

  // Global Host actions bridge
  window.conferHostAdmit = (id) => sendWsMessage({ type: 'admit_participant', participant_id: id });
  window.conferHostReject = (id) => sendWsMessage({ type: 'reject_participant', participant_id: id });
  window.conferHostKick = (id) => sendWsMessage({ type: 'host_action', action: 'kick', target_participant_id: id });

  // --- Polls UI & Engine ---
  function createPoll() {
    const question = els.pollQuestionInput.value.trim();
    const optInputs = els.pollOptionsInputs.querySelectorAll('.poll-opt-input');
    const options = Array.from(optInputs).map(i => i.value.trim()).filter(Boolean);

    if (!question || options.length < 2) return alert("Please enter question and at least 2 options.");

    sendWsMessage({
      type: 'create_poll',
      poll_id: crypto.randomUUID(),
      question,
      options,
      multi_choice: els.pollMultichoiceCheck.checked,
      is_anonymous: els.pollAnonCheck.checked
    });

    els.pollQuestionInput.value = '';
    optInputs.forEach(i => i.value = '');
  }

  function renderPolls() {
    els.activePollsContainer.innerHTML = '';
    state.polls.forEach(poll => {
      const card = document.createElement('div');
      card.className = 'poll-card';
      const total = poll.total_votes || 1;

      let optionsHtml = '';
      poll.options.forEach((opt, idx) => {
        const pct = Math.round((opt.vote_count / total) * 100);
        optionsHtml += `
          <div class="poll-option-row">
            <div style="display:flex;justify-content:space-between;font-size:12px;">
              <button class="btn-text-sm" onclick="window.conferVotePoll('${poll.id}', ${idx})">${opt.text}</button>
              <span>${pct}% (${opt.vote_count})</span>
            </div>
            <div class="poll-progress-bar">
              <div class="poll-progress-fill" style="width:${pct}%"></div>
            </div>
          </div>
        `;
      });

      card.innerHTML = `
        <div class="poll-question">${poll.question}</div>
        ${optionsHtml}
        ${state.user.role === 'host' && !poll.is_closed ? `<button class="btn-xs btn-danger" style="align-self:flex-end;margin-top:6px;" onclick="window.conferClosePoll('${poll.id}')">Close Poll</button>` : ''}
      `;
      els.activePollsContainer.appendChild(card);
    });
  }

  window.conferVotePoll = (pollId, optIdx) => sendWsMessage({ type: 'vote_poll', poll_id: pollId, selected_options: [optIdx] });
  window.conferClosePoll = (pollId) => sendWsMessage({ type: 'close_poll', poll_id: pollId });

  // --- Collaborative Whiteboard (HTML5 Canvas) ---
  let isDrawing = false;
  let currentStroke = [];
  let currentTool = 'pen';

  function setupWhiteboard() {
    const canvas = els.whiteboardCanvas;
    const ctx = canvas.getContext('2d');

    document.querySelectorAll('.wb-tool-btn[data-tool]').forEach(btn => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.wb-tool-btn[data-tool]').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        currentTool = btn.dataset.tool;
      });
    });

    els.wbUndoBtn.addEventListener('click', () => {
      state.whiteboardStrokes.pop();
      redrawAllStrokes();
    });

    els.wbClearBtn.addEventListener('click', () => {
      sendWsMessage({ type: 'whiteboard_clear' });
      clearLocalWhiteboard();
    });

    canvas.addEventListener('mousedown', (e) => {
      isDrawing = true;
      const rect = canvas.getBoundingClientRect();
      currentStroke = [{ x: e.clientX - rect.left, y: e.clientY - rect.top }];
    });

    canvas.addEventListener('mousemove', (e) => {
      if (!isDrawing) return;
      const rect = canvas.getBoundingClientRect();
      const pt = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      currentStroke.push(pt);

      ctx.strokeStyle = currentTool === 'eraser' ? '#121418' : els.wbColorPicker.value;
      ctx.lineWidth = parseInt(els.wbStrokeWidth.value, 10);
      ctx.lineCap = 'round';

      ctx.beginPath();
      const prev = currentStroke[currentStroke.length - 2];
      ctx.moveTo(prev.x, prev.y);
      ctx.lineTo(pt.x, pt.y);
      ctx.stroke();
    });

    canvas.addEventListener('mouseup', () => {
      if (!isDrawing) return;
      isDrawing = false;
      if (currentStroke.length > 1) {
        const strokeDto = {
          id: crypto.randomUUID(),
          tool: currentTool,
          color: hexToRgb(els.wbColorPicker.value),
          stroke_width: parseInt(els.wbStrokeWidth.value, 10),
          points: currentStroke.map(p => [p.x, p.y])
        };
        state.whiteboardStrokes.push(strokeDto);
        sendWsMessage({ type: 'whiteboard_stroke', stroke: strokeDto });
      }
    });

    window.addEventListener('resize', resizeWhiteboard);
  }

  function resizeWhiteboard() {
    const canvas = els.whiteboardCanvas;
    canvas.width = canvas.parentElement.clientWidth;
    canvas.height = canvas.parentElement.clientHeight - 48;
    redrawAllStrokes();
  }

  function redrawAllStrokes() {
    const canvas = els.whiteboardCanvas;
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    state.whiteboardStrokes.forEach(s => drawStrokeOnCanvas(s));
  }

  function drawStrokeOnCanvas(stroke) {
    const canvas = els.whiteboardCanvas;
    const ctx = canvas.getContext('2d');
    if (!stroke.points || stroke.points.length < 2) return;

    ctx.strokeStyle = stroke.tool === 'eraser' ? '#121418' : `rgb(${stroke.color?.r ?? 255}, ${stroke.color?.g ?? 255}, ${stroke.color?.b ?? 255})`;
    ctx.lineWidth = stroke.stroke_width || 3;
    ctx.lineCap = 'round';

    ctx.beginPath();
    ctx.moveTo(stroke.points[0][0], stroke.points[0][1]);
    for (let i = 1; i < stroke.points.length; i++) {
      ctx.lineTo(stroke.points[i][0], stroke.points[i][1]);
    }
    ctx.stroke();
  }

  function clearLocalWhiteboard() {
    state.whiteboardStrokes = [];
    const canvas = els.whiteboardCanvas;
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, canvas.width, canvas.height);
  }

  function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16),
      a: 255
    } : { r: 255, g: 255, b: 255, a: 255 };
  }

  // --- Watermark & Host Policies ---
  function updateWatermarkPattern() {
    els.watermarkPattern.innerHTML = '';
    const text = `${state.user.name} • ${state.user.email}`;
    for (let i = 0; i < 9; i++) {
      const tag = document.createElement('div');
      tag.className = 'watermark-tag';
      tag.textContent = text;
      els.watermarkPattern.appendChild(tag);
    }
    els.watermarkOverlay.style.display = state.isWatermarkEnabled ? 'grid' : 'none';
  }

  function updateHostPolicies() {
    const policy = {
      is_locked: els.policyLockCheck.checked,
      waiting_room_enabled: els.policyWaitingCheck.checked,
      allow_screen_share: els.policyScreenshareCheck.checked,
      allow_chat: els.policyChatCheck.checked,
      allow_unmute: els.policyUnmuteCheck.checked,
      watermark_enabled: els.policyWatermarkCheck.checked
    };
    sendWsMessage({ type: 'update_policy', policy });
  }

  function applyPolicies() {
    if (state.meetingPolicy) {
      els.policyLockCheck.checked = state.meetingPolicy.is_locked;
      els.policyWaitingCheck.checked = state.meetingPolicy.waiting_room_enabled;
      els.policyScreenshareCheck.checked = state.meetingPolicy.allow_screen_share;
      els.policyChatCheck.checked = state.meetingPolicy.allow_chat;
      els.policyUnmuteCheck.checked = state.meetingPolicy.allow_unmute;
      els.policyWatermarkCheck.checked = state.meetingPolicy.watermark_enabled;

      if (!state.meetingPolicy.allow_screen_share && state.user.role !== 'host') {
        els.dockScreenshareBtn.disabled = true;
      } else {
        els.dockScreenshareBtn.disabled = false;
      }
    }
    updateWatermarkPattern();
  }

  // --- i18n ---
  function updateI18n() {
    const t = i18n[state.lang];
    document.getElementById('i18n-lobby-title').textContent = t.lobbyTitle;
    document.getElementById('i18n-lobby-subtitle').textContent = t.lobbySubtitle;
    document.getElementById('i18n-label-name').textContent = t.labelName;
    document.getElementById('i18n-create-title').textContent = t.createTitle;
    document.getElementById('create-meeting-btn').textContent = t.btnCreate;
    document.getElementById('i18n-join-title').textContent = t.joinTitle;
    document.getElementById('join-meeting-btn').textContent = t.btnJoin;
    document.getElementById('i18n-waiting-title').textContent = t.waitingTitle;

    els.dockMicBtn.querySelector('span').textContent = state.isMicMuted ? t.unmute : t.mute;
    els.dockCamBtn.querySelector('span').textContent = t.camera;
    els.dockScreenshareBtn.querySelector('span').textContent = t.share;
    els.dockWhiteboardBtn.querySelector('span').textContent = t.whiteboard;
    els.dockHandBtn.querySelector('span').textContent = t.hand;
    els.dockLeaveBtn.querySelector('span').textContent = t.leave;
  }

  // Start Application
  window.addEventListener('DOMContentLoaded', init);
})();
