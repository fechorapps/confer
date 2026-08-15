# 🔌 Confer REST API & WebSocket Signaling Specification

Welcome to the **Confer Developer Reference**. This document outlines the REST endpoints and WebSocket signaling protocol for integrating Confer into your platforms.

---

## Base URLs
- **REST API**: `http://localhost:5000/api` (or `https://meet.confer.local/api`)
- **WebSocket Signaling**: `ws://localhost:5000/v1/signal` (subprotocol: `confer.v1`)

---

## 1. Authentication & Single Sign-On (SSO)

### `POST /api/auth/dev-login`
Development login endpoint returning a JWT bearer token.
```json
// Request
{
  "email": "developer@company.com",
  "displayName": "Jane Developer"
}

// Response (200 OK)
{
  "userId": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "email": "developer@company.com",
  "displayName": "Jane Developer",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### `GET /api/auth/sso/providers`
Lists configured enterprise identity providers (Google, Microsoft, Okta, SAML 2.0).
```json
// Response (200 OK)
[
  {
    "providerType": 1,
    "name": "google",
    "displayName": "Google Workspace",
    "iconUrl": "https://auth.confer.local/icons/google.svg",
    "isEnabled": true
  },
  {
    "providerType": 2,
    "name": "microsoft",
    "displayName": "Microsoft Entra ID (Azure AD)",
    "iconUrl": "https://auth.confer.local/icons/microsoft.svg",
    "isEnabled": true
  }
]
```

### `GET /api/auth/sso/{provider}/authorize`
Generates an OAuth2/OIDC PKCE authorization URL.
```json
// Response (200 OK)
{
  "authorizationUrl": "https://accounts.google.com/o/oauth2/v2/auth?client_id=...&code_challenge=...",
  "state": "a1b2c3d4e5f6",
  "codeVerifier": "v1w2x3y4z5..."
}
```

---

## 2. Meetings & Sessions

### `POST /api/meetings`
Creates a new meeting room.
```json
// Request
{
  "title": "Quarterly Business Review",
  "ownerId": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "maxParticipants": 100,
  "isWaitingRoomEnabled": true,
  "isWatermarkEnabled": true,
  "policy": {
    "allowScreenShare": true,
    "allowChat": true,
    "allowUnmuteSelf": true,
    "muteOnEntry": false,
    "allowRename": true
  }
}

// Response (200 OK)
{
  "id": "e5b8d2c4-f0a1-42e6-9538-3b1a2d5e7c8f",
  "joinCode": "729415",
  "title": "Quarterly Business Review",
  "ownerId": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "maxParticipants": 100
}
```

### `POST /api/meetings/{id}/join`
Joins an active meeting room and returns room token.
```json
// Request
{
  "userId": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "displayName": "Jane Developer",
  "clientInfo": "Confer Web 1.0"
}

// Response (200 OK)
{
  "meetingId": "e5b8d2c4-f0a1-42e6-9538-3b1a2d5e7c8f",
  "title": "Quarterly Business Review",
  "participantId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "role": "host",
  "isLocked": false,
  "wsUrl": "ws://localhost:5000/v1/signal",
  "roomToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "status": "admitted",
  "isWaitingRoom": false
}
```

---

## 3. Telephony & PSTN Dial-In

### `GET /api/meetings/{id}/telephony`
Returns PSTN phone dial-in coordinates and conference PIN.
```json
// Response (200 OK)
{
  "meetingId": "e5b8d2c4-f0a1-42e6-9538-3b1a2d5e7c8f",
  "title": "Quarterly Business Review",
  "phoneNumber": "+1 (888) 555-CONFER",
  "conferencePin": "729415",
  "directSipUri": "sip:729415@sip.confer.local",
  "globalDialInNumbers": [
    "+1 (888) 555-CONFER (US Toll Free)",
    "+44 20 7946 0991 (London, UK)",
    "+52 55 4169 8830 (Mexico City, Mexico)"
  ]
}
```

---

## 4. AI Meeting Summaries & Actions

### `POST /api/meetings/{id}/summary`
Generates or retrieves executive summary, key decisions, and action items.
```json
// Response (200 OK)
{
  "id": "1fa85f64-5717-4562-b3fc-2c963f66afa6",
  "meetingId": "e5b8d2c4-f0a1-42e6-9538-3b1a2d5e7c8f",
  "overview": "The team aligned on the Q3 release milestones...",
  "keyDecisions": [
    "Approved deployment of Kubernetes Helm chart to production",
    "Enabled SFrame E2EE by default for enterprise tier"
  ],
  "actionItems": [
    { "title": "Run security audit on TURN cluster", "assignee": "Alice", "status": "Pending" },
    { "title": "Tag v1.0.0 release in GitHub", "assignee": "Bob", "status": "Completed" }
  ],
  "durationMinutes": 45,
  "participantCount": 8
}
```
