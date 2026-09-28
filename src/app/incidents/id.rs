use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::Form,
        error::{RouterErrorExt, SeeOther, see_other},
        href, page, path_param, route,
    },
    runtime::connected,
    view::{View, component, emit, live, view},
};

use crate::{
    app::db,
    models::{Incident, IncidentUpdate},
    state::{AppState, notify},
};

path_param!(pub id);

#[page]
pub(crate) async fn page(cx: &Cx) -> Result<impl View> {
    let public_id: &str = path_param::<Id>(cx);
    let incident = Incident::filter(Incident::fields().public_id().eq(public_id.to_owned()))
        .first()
        .exec(&mut db(cx))
        .await?
        .ok_or_not_found()?;

    Ok(view! {
        <a class="font-mono text-xs text-primary" href="/incidents">"← INCIDENT LOG"</a>
        <article class="mt-6 border-b border-white/15 pb-8">
            <p class="break-all font-mono text-xs uppercase tracking-widest text-primary">"INCIDENT / " (incident.public_id)</p>
            <h1 class="mt-3 max-w-4xl text-4xl leading-tight tracking-tight md:text-5xl">(&incident.title)</h1>
            <p class="mt-4 max-w-3xl text-lg opacity-65">(&incident.summary)</p>
        </article>
        <div class="mt-8 grid gap-10 lg:grid-cols-[1fr_320px]">
            <section>
                <div class="flex items-center justify-between gap-4">
                    <div>
                        <p class="font-mono text-xs uppercase tracking-widest text-primary">"LIVE TIMELINE"</p>
                        <h2 class="mt-2 text-2xl">"Operator updates"</h2>
                    </div>
                    <span class="font-mono text-xs opacity-50">"WEBSOCKET"</span>
                </div>
                <div class="mt-6">timeline(incident_id: incident.id)</div>
            </section>
            <aside class="h-fit border border-white/15 bg-white/[.03] p-5">
                <p class="font-mono text-xs uppercase tracking-widest text-primary">"Post an update"</p>
                <form method="post" action=(format!("/incidents/{public_id}")) class="mt-4 grid gap-3">
                    <select class="border border-white/20 bg-[#171f1a] px-3 py-3" name="status">
                        <option value="Investigating">"Investigating"</option>
                        <option value="Identified">"Identified"</option>
                        <option value="Monitoring">"Monitoring"</option>
                        <option value="Resolved">"Resolved"</option>
                    </select>
                    <textarea class="min-h-32 border border-white/20 bg-transparent px-3 py-3" name="message" maxlength="1000" placeholder="Share what changed..." required=""></textarea>
                    <button class="bg-primary px-4 py-3 text-left font-mono text-sm text-black" type="submit">"Publish update →"</button>
                </form>
            </aside>
        </div>
    })
}

#[component]
async fn timeline(cx: &Cx, incident_id: i64) -> Result<impl View> {
    Ok(live! {
        let mut changed = app_context::<AppState>(cx).changed.subscribe();
        loop {
            let mut updates = IncidentUpdate::all().exec(&mut db(cx)).await?;
            updates.retain(|update| update.incident_id == incident_id);
            updates.reverse();
            let token = emit! {
                if updates.is_empty() {
                    <p class="border border-dashed border-white/20 p-6 opacity-60">"No updates recorded."</p>
                } else {
                    <ol class="border-l border-primary/40 pl-5">
                        for update in updates.iter() {
                            <li class="relative mb-7">
                                <span class="absolute -left-[25px] top-1 h-2 w-2 rounded-full bg-primary"></span>
                                <p class="font-mono text-xs uppercase text-primary">(&update.status)</p>
                                <p class="mt-2 leading-7 opacity-80">(&update.message)</p>
                                <p class="mt-2 break-all font-mono text-[10px] opacity-40">"UPDATE / " (&update.public_id)</p>
                            </li>
                        }
                    </ol>
                }
            }?;
            if !connected(cx) {
                break Ok(token);
            }
            let _ = changed.recv().await;
        }
    })
}

#[derive(Deserialize)]
struct UpdateForm {
    status: String,
    message: String,
}

#[route(POST)]
async fn add_update(cx: &Cx, Form(form): Form<UpdateForm>) -> Result<SeeOther> {
    let public_id: &str = path_param::<Id>(cx);
    let status = form.status.trim();
    let message = form.message.trim();
    if ["Investigating", "Identified", "Monitoring", "Resolved"].contains(&status)
        && !message.is_empty()
    {
        let incident = Incident::filter(Incident::fields().public_id().eq(public_id.to_owned()))
            .first()
            .exec(&mut db(cx))
            .await?
            .ok_or_not_found()?;
        toasty::create!(IncidentUpdate {
            incident_id: incident.id,
            public_id: uuid::Uuid::now_v7().to_string(),
            status: status.to_owned(),
            message: message.to_owned(),
        })
        .exec(&mut db(cx))
        .await?;
        notify(cx);
    }
    Ok(see_other(href!(page, Id(public_id.to_owned())).resolve(cx)))
}
