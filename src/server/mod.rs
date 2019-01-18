//! Web server utilities.
mod job;
mod state;
use actix_web::{http::Method, App};

/// Returns an actix-web app for handling jobs.
fn job_app(state: state::State) -> App<state::State> {
    App::with_state(state)
        .route("/", Method::GET, job::status)
        .route("/", Method::HEAD, job::status)
        .route("/", Method::POST, job::start)
        .resource("/{job}", |r| r.method(Method::DELETE).with(job::halt))
        .resource("/{job}/resume", |r| {
            r.method(Method::POST).with(job::resume)
        })
}

/// Returns an actix-web app for handling protocols.
fn protocol_app(state: state::State) -> App<state::State> {
    App::with_state(state)
}

fn state() -> state::State {
    unimplemented!()
}

/// Returns the list of actix-web apps to be used with the server.
pub fn apps() -> Vec<App<state::State>> {
    let state = state();
    vec![job_app(state.clone()), protocol_app(state.clone())]
}
