mod app;
mod migrations;
mod models;
mod state;

use toasty::Db;
use tokio::sync::broadcast;

use crate::models::{Incident, IncidentUpdate};

#[tokio::main]
async fn main() {
    let initialize_schema = !std::path::Path::new("dispatch.sqlite").exists();
    if !initialize_schema {
        migrations::ensure_public_ids("dispatch.sqlite").unwrap();
    }
    let db = Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite:dispatch.sqlite")
        .await
        .unwrap();
    if initialize_schema {
        db.push_schema().await.unwrap();
        migrations::ensure_public_ids("dispatch.sqlite").unwrap();
    }

    if Incident::all()
        .exec(&mut db.clone())
        .await
        .unwrap()
        .is_empty()
    {
        let incident = toasty::create!(Incident {
            public_id: uuid::Uuid::now_v7().to_string(),
            title: "Example: Search requests intermittently stall".to_owned(),
            summary: "A sample incident to explore the detail page and live status timeline. This is demo data, not a real service alert.".to_owned(),
        })
        .exec(&mut db.clone())
        .await
        .unwrap();
        for (status, message) in [
            (
                "Investigating",
                "Example update: reports of slower search requests have been received.",
            ),
            (
                "Identified",
                "Example update: a slow index refresh is delaying a portion of requests.",
            ),
            (
                "Monitoring",
                "Example update: the refresh has been paused; latency is returning to normal.",
            ),
            (
                "Resolved",
                "Example update: search latency has returned to baseline.",
            ),
        ] {
            toasty::create!(IncidentUpdate {
                incident_id: incident.id,
                public_id: uuid::Uuid::now_v7().to_string(),
                status: status.to_owned(),
                message: message.to_owned(),
            })
            .exec(&mut db.clone())
            .await
            .unwrap();
        }
    }

    let (changed, _) = broadcast::channel(64);
    topcoat::start(app::router(db, state::AppState { changed }))
        .await
        .unwrap();
}
