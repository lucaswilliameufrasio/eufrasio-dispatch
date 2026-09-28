pub(crate) mod id;
pub(crate) mod new;

use topcoat::{
    Result,
    router::page,
    view::{View, view},
};

use crate::{
    app::db,
    models::{Incident, IncidentUpdate},
};

#[page]
async fn page(cx: &topcoat::context::Cx) -> Result<impl View> {
    let mut incidents = Incident::all().exec(&mut db(cx)).await?;
    incidents.reverse();
    let updates = IncidentUpdate::all().exec(&mut db(cx)).await?;

    Ok(view! {
        <div class="flex flex-wrap items-end justify-between gap-5">
            <div>
                <p class="font-mono text-xs uppercase tracking-[.2em] text-primary">"EUFRASIO / DISPATCH"</p>
                <h1 class="mt-3 text-4xl tracking-tight">"Incident log"</h1>
                <p class="mt-2 opacity-60">"Every report and operator update, in one place."</p>
            </div>
            <a class="bg-primary px-4 py-3 font-mono text-sm text-black" href="/incidents/new">"Open incident +"</a>
        </div>
        if incidents.is_empty() {
            <div class="mt-8 border border-dashed border-white/20 p-10 text-center">
                <h2 class="text-xl">"No incidents yet"</h2>
                <p class="mt-2 opacity-60">"Create one to start a public timeline."</p>
                <a class="mt-5 inline-block text-primary" href="/incidents/new">"Create the first incident →"</a>
            </div>
        } else {
            <div class="mt-8 grid gap-3">
                for incident in incidents.iter() {
                    let latest = updates.iter().filter(|u| u.incident_id == incident.id).max_by_key(|u| u.id);
                    let short_id = incident.public_id.get(..8).unwrap_or(&incident.public_id);
                    <a class="grid gap-4 border border-white/15 p-5 hover:border-primary/60 sm:grid-cols-[1fr_auto]" href=(format!("/incidents/{}", incident.public_id.as_str()))>
                        <div>
                            <p class="font-mono text-xs text-primary" title=(incident.public_id.as_str())>"INCIDENT / " (short_id)</p>
                            <h2 class="mt-2 text-xl">(&incident.title)</h2>
                            <p class="mt-2 max-w-3xl text-sm opacity-60">(&incident.summary)</p>
                        </div>
                        <div class="sm:text-right">
                            if let Some(update) = latest {
                                <span class="border border-primary/40 px-2 py-1 font-mono text-xs text-primary">(&update.status)</span>
                                <p class="mt-3 text-xs opacity-50">"" (updates.iter().filter(|u| u.incident_id == incident.id).count()) " updates"</p>
                            } else {
                                <span class="border border-white/20 px-2 py-1 font-mono text-xs opacity-50">"No updates"</span>
                            }
                        </div>
                    </a>
                }
            </div>
        }
    })
}
