use tokio::sync::broadcast;
use topcoat::context::{Cx, app_context};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) changed: broadcast::Sender<()>,
}

pub(crate) fn notify(cx: &Cx) {
    let _ = app_context::<AppState>(cx).changed.send(());
}
