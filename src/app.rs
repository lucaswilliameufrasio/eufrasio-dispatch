pub(crate) mod incidents;

use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context, try_request_context},
    router::{
        Body, LayerFn, LayerFuture, Next, Router, RouterBuilderDiscoverExt, Slot, StatusCode,
        error::{
            BadRequestError, ContentTooLargeError, ForbiddenError, MethodNotAllowedError,
            NotFoundError, ServiceUnavailableError, TooManyRequestsError, UnauthorizedError,
            rewrite,
        },
        layout, module_router, not_found, page,
    },
    runtime::RouterBuilderRuntimeExt,
    tailwind,
    view::{View, error_boundary, view},
};

use crate::{models::Incident, state::AppState};

not_found!("/");

pub(crate) fn router(db: Db, state: AppState) -> Router {
    module_router!()
        .layer(LayerFn::new(None::<&str>, error_layer))
        .discover()
        .assets(AssetBundle::load().unwrap())
        .app_context(db)
        .app_context(state)
        .runtime()
        .build()
}

pub(crate) fn db(cx: &Cx) -> Db {
    app_context::<Db>(cx).clone()
}

#[derive(Clone, Copy)]
struct ErrorPage {
    status: StatusCode,
    code: &'static str,
    title: &'static str,
    message: &'static str,
}

const NOT_FOUND_PAGE: ErrorPage = ErrorPage {
    status: StatusCode::NOT_FOUND,
    code: "404",
    title: "That signal goes nowhere.",
    message: "No incident or page exists at this address. The old numeric links were retired; incident links now use UUIDv7.",
};

fn describe_error(error: &topcoat::Error) -> ErrorPage {
    if error.downcast_ref::<NotFoundError>().is_some() {
        NOT_FOUND_PAGE
    } else if error.downcast_ref::<BadRequestError>().is_some() {
        ErrorPage {
            status: StatusCode::BAD_REQUEST,
            code: "400",
            title: "That request could not be read.",
            message: "Check the submitted fields and try again.",
        }
    } else if error.downcast_ref::<UnauthorizedError>().is_some() {
        ErrorPage {
            status: StatusCode::UNAUTHORIZED,
            code: "401",
            title: "This desk needs a valid session.",
            message: "Sign in and try that request again.",
        }
    } else if error.downcast_ref::<ForbiddenError>().is_some() {
        ErrorPage {
            status: StatusCode::FORBIDDEN,
            code: "403",
            title: "You cannot change this incident.",
            message: "The request is not permitted.",
        }
    } else if error.downcast_ref::<MethodNotAllowedError>().is_some() {
        ErrorPage {
            status: StatusCode::METHOD_NOT_ALLOWED,
            code: "405",
            title: "That action is not available here.",
            message: "Use one of the actions shown on this page.",
        }
    } else if error.downcast_ref::<ContentTooLargeError>().is_some() {
        ErrorPage {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "413",
            title: "That report is too large.",
            message: "Shorten the submitted details and try again.",
        }
    } else if error.downcast_ref::<TooManyRequestsError>().is_some() {
        ErrorPage {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "429",
            title: "The desk is receiving too many requests.",
            message: "Wait a moment, then try again.",
        }
    } else if error.downcast_ref::<ServiceUnavailableError>().is_some() {
        ErrorPage {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "503",
            title: "The desk is temporarily offline.",
            message: "Please try again shortly.",
        }
    } else {
        ErrorPage {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "500",
            title: "Something broke on our side.",
            message: "The incident desk hit an unexpected error. The details stay in the server logs.",
        }
    }
}

fn error_content(cx: &Cx, error: ErrorPage) -> impl View {
    view! { cx =>
        (error.status)
        <section class="mx-auto max-w-3xl border border-white/15 bg-white/[.03] p-8 md:p-12">
            <p class="font-mono text-xs uppercase tracking-[.2em] text-primary">"EUFRASIO / DISPATCH · ERROR " (error.code)</p>
            <h1 class="mt-6 text-4xl leading-tight tracking-[-.04em] md:text-6xl">(error.title)</h1>
            <p class="mt-5 max-w-2xl text-lg opacity-65">(error.message)</p>
            <div class="mt-8 flex flex-wrap gap-3">
                <a class="bg-primary px-5 py-3 font-mono text-sm text-black" href="/">"Back to overview"</a>
                <a class="border border-white/20 px-5 py-3 font-mono text-sm" href="/incidents">"Incident log"</a>
            </div>
        </section>
    }
}

fn error_layer<'a>(cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
    Box::pin(async move {
        match next.run(cx, body).await {
            Ok(response) => Ok(response),
            Err(error) => {
                let page = describe_error(&error);
                Err(rewrite("/_errors", Body::empty())
                    .method(topcoat::router::Method::GET)
                    .with(page)
                    .into())
            }
        }
    })
}

#[page("/_errors")]
async fn error_page(cx: &Cx) -> Result<impl View> {
    let page = try_request_context::<ErrorPage>(cx)
        .copied()
        .unwrap_or(NOT_FOUND_PAGE);
    Ok(error_content(cx, page))
}

#[layout]
async fn shell(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="theme-color" content="#111714">
                <title>"Eufrasio Dispatch"</title>
                topcoat::runtime::script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body class="min-h-screen">
                <header class="border-b border-white/15">
                    <nav class="mx-auto flex max-w-6xl items-center gap-7 px-6 py-5 font-mono text-xs uppercase tracking-[.16em]">
                        <a class="font-bold text-primary" href="/">"EUFRASIO / DISPATCH"</a>
                        <a class="opacity-70 hover:text-primary" href="/incidents">"Incidents"</a>
                    </nav>
                </header>
                <main class="mx-auto max-w-6xl px-6 py-10">
                    error_boundary(
                        fallback: |error| Ok(error_content(cx, describe_error(&error))),
                        (slot)
                    )
                </main>
                <footer class="mx-auto max-w-6xl px-6 py-8 font-mono text-xs opacity-50">"A small incident desk, built with Topcoat + Toasty."</footer>
            </body>
        </html>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let incidents = Incident::all().exec(&mut db(cx)).await?;
    let count = incidents.len();

    Ok(view! {
        <div class="grid gap-10 lg:grid-cols-[1fr_340px]">
            <section>
                <p class="font-mono text-xs uppercase tracking-[.2em] text-primary">"EUFRASIO / OPERATIONS"</p>
                <h1 class="mt-5 max-w-3xl text-5xl leading-[.96] tracking-[-.06em] md:text-7xl">
                    "A clear record"<br>"when things break."
                </h1>
                <p class="mt-6 max-w-2xl text-lg opacity-65">
                    "A small, public incident desk. Each report has its own page and a live-updating operator timeline."
                </p>
                <div class="mt-8 flex flex-wrap gap-3">
                    <a class="bg-primary px-5 py-3 font-mono text-sm text-black" href="/incidents/new">"Open an incident →"</a>
                    <a class="border border-white/20 px-5 py-3 font-mono text-sm" href="/incidents">"Browse incident log"</a>
                </div>
            </section>
            <aside class="border border-white/15 bg-white/[.03] p-6">
                <p class="font-mono text-xs uppercase tracking-widest text-primary">"The desk"</p>
                <p class="mt-8 text-5xl">(count)</p>
                <p class="mt-2 text-sm opacity-60">"incidents recorded"</p>
                <div class="mt-6 border-t border-white/10 pt-5">
                    <p class="text-sm opacity-70">"Updates are stored in SQLite via Toasty and pushed live to connected readers."</p>
                </div>
            </aside>
        </div>
        <section class="mt-16 border-t border-white/15 pt-8">
            <div class="flex items-center justify-between gap-4">
                <h2 class="text-2xl">"Latest incidents"</h2>
                <a class="font-mono text-xs text-primary" href="/incidents">"ALL INCIDENTS →"</a>
            </div>
            if incidents.is_empty() {
                <p class="mt-6 border border-dashed border-white/20 p-6 opacity-60">"The log is quiet. Open an incident to start a timeline."</p>
            } else {
                <div class="mt-5 grid gap-3 sm:grid-cols-2">
                    for incident in incidents.iter().rev().take(4) {
                        let short_id = incident.public_id.get(..8).unwrap_or(&incident.public_id);
                        <a class="border border-white/15 p-5 hover:border-primary/60" href=(format!("/incidents/{}", incident.public_id.as_str()))>
                            <p class="font-mono text-xs text-primary" title=(incident.public_id.as_str())>"INCIDENT / " (short_id)</p>
                            <h3 class="mt-2 text-xl">(&incident.title)</h3>
                            <p class="mt-2 line-clamp-2 text-sm opacity-60">(&incident.summary)</p>
                        </a>
                    }
                </div>
            }
        </section>
    })
}

#[cfg(test)]
mod tests {
    use super::{NOT_FOUND_PAGE, describe_error};
    use topcoat::router::{StatusCode, error::bad_request};

    #[test]
    fn custom_error_mapping_keeps_status_and_does_not_expose_internal_messages() {
        let not_found: topcoat::Error = topcoat::router::error::not_found().into();
        assert_eq!(describe_error(&not_found).status, StatusCode::NOT_FOUND);
        assert_eq!(describe_error(&not_found).title, NOT_FOUND_PAGE.title);

        let bad_request: topcoat::Error = bad_request("private form detail").into();
        let page = describe_error(&bad_request);
        assert_eq!(page.status, StatusCode::BAD_REQUEST);
        assert!(!page.message.contains("private form detail"));

        let internal = topcoat::Error::msg("database password should not reach the page");
        let page = describe_error(&internal);
        assert_eq!(page.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!page.message.contains("database password"));
    }
}
