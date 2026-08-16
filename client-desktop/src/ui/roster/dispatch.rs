use crate::app::ConferApp;
use uuid::Uuid;

/// Pending host actions collected while rendering the roster, applied to
/// `app` once at the end of `render_roster`.
#[derive(Default)]
pub(super) struct RosterActions {
    pub admit: Option<Uuid>,
    pub reject: Option<Uuid>,
    pub admit_all: bool,
    pub mute: Option<Uuid>,
    pub mute_all: bool,
    pub kick: Option<(Uuid, String)>,
}

/// Dispatch pending host actions.
pub(super) fn dispatch_roster_actions(app: &mut ConferApp, actions: RosterActions) {
    if let Some(id) = actions.admit {
        app.admit_participant(id);
    }
    if let Some(id) = actions.reject {
        app.reject_participant(id);
    }
    if actions.admit_all {
        app.admit_all_waiting();
    }
    if let Some(id) = actions.mute {
        app.host_mute_participant(id);
    }
    if let Some(target) = actions.kick {
        app.kick_confirmation_target = Some(target);
    }
    if actions.mute_all {
        let to_mute: Vec<Uuid> = app
            .roster
            .iter()
            .filter(|p| !p.is_audio_muted)
            .map(|p| p.user_id)
            .collect();
        for id in to_mute {
            app.host_mute_participant(id);
        }
    }
}
