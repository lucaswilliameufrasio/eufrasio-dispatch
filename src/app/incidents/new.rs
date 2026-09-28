use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, page, route,
    },
    view::{View, view},
};

use crate::{
    app::db,
    models::{Incident, IncidentUpdate},
    state::notify,
};

#[page]
async fn page() -> Result<impl View> {
    Ok(view! {
        <a class="font-mono text-xs text-primary" href="/incidents">"← INCIDENT LOG"</a>
        <div class="mx-auto mt-8 max-w-2xl">
            <p class="font-mono text-xs uppercase tracking-[.2em] text-primary">"NEW REPORT"</p>
            <h1 class="mt-3 text-4xl">"Open an incident"</h1>
            <p class="mt-3 opacity-60">"Create a public record. You can add status updates from its detail page."</p>
            <form method="post" action="/incidents/new" class="mt-8 grid gap-5 border border-white/15 bg-white/[.03] p-6">
                <label class="grid gap-2 text-sm">"Incident title"
                    <input class="border border-white/20 bg-transparent px-3 py-3" name="title" maxlength="120" placeholder="API requests timing out" required="">
                </label>
                <label class="grid gap-2 text-sm">"Impact summary"
                    <textarea class="min-h-36 border border-white/20 bg-transparent px-3 py-3" name="summary" maxlength="1000" placeholder="What is affected? What should users know?" required=""></textarea>
                </label>
                <div class="flex justify-end gap-3">
                    <a class="border border-white/20 px-4 py-3 text-sm" href="/incidents">"Cancel"</a>
                    <button class="bg-primary px-5 py-3 font-mono text-sm text-black" type="submit">"Publish incident"</button>
                </div>
            </form>
        </div>
    })
}

#[derive(Deserialize)]
struct IncidentForm {
    title: String,
    summary: String,
}

#[route(POST)]
async fn create(cx: &Cx, Form(form): Form<IncidentForm>) -> Result<SeeOther> {
    let title = form.title.trim();
    let summary = form.summary.trim();
    let destination = if !title.is_empty() && !summary.is_empty() {
        let incident = toasty::create!(Incident {
            public_id: uuid::Uuid::now_v7().to_string(),
            title: title.to_owned(),
            summary: summary.to_owned(),
        })
        .exec(&mut db(cx))
        .await?;
        toasty::create!(IncidentUpdate {
            incident_id: incident.id,
            public_id: uuid::Uuid::now_v7().to_string(),
            status: "Investigating".to_owned(),
            message: "Investigation is underway.".to_owned(),
        })
        .exec(&mut db(cx))
        .await?;
        notify(cx);
        href!(super::id::page, super::id::Id(incident.public_id)).resolve(cx)
    } else {
        href!(super::page).resolve(cx)
    };
    Ok(see_other(destination))
}
