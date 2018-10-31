use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_web::{error, server, App, Error, HttpRequest, HttpResponse, Responder};

use deoxy::communication::{Action, Coordinator};
use deoxy::config::Config;

use serde_json;

struct AppState {
    pub config: Config,
    pub coord: Coordinator,
}

struct ConfigExt {
    config: Config,
}

impl From<Config> for ConfigExt {
    fn from(config: Config) -> Self {
        Self { config }
    }
}

impl Responder for ConfigExt {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<Self::Item, Self::Error> {
        let body = serde_json::to_string(&self.config)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

/// Starts `deoxy-www` (on the current thread).
///
/// This method starts up the event loop and binds a web server to `127.0.0.1:1957` (subject to
/// change).
///
/// ### Notes
///
/// This method will not exit until the web server has terminated (for whatever reason).
/// Once it has exited, all motors and pumps will be inactive (closed or off).
/// Eventually, this will start a TCP server instead, with the web server communicating over TCP to
/// `deoxyd`. However, with the current constraints, we are punting on that for now.

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn start(config: Config) {
    let coord = Coordinator::from(&config);
    let state = Arc::new(Mutex::new(AppState { config, coord }));
    server::new(move || {
        App::with_state(state.clone())
            .resource("/", |r| r.f(hello))
            .resource("/config", |r| r.f(cfg))
            .resource("/demo", |r| r.f(demo))
    })
    .bind("127.0.0.1:1957")
    .expect("Could not bind server to port 1957.")
    .run();
}

fn hello(_req: &HttpRequest<Arc<Mutex<AppState>>>) -> impl Responder {
    "Hello, world!"
}

fn cfg(req: &HttpRequest<Arc<Mutex<AppState>>>) -> ConfigExt {
    let state = req.state().lock().unwrap();
    state.config.clone().into()
}

fn demo(req: &HttpRequest<Arc<Mutex<AppState>>>) -> impl Responder {
    let mgr = &req.state().lock().unwrap().coord;
    mgr.channels[0]
        .send(Action::Open(Duration::from_millis(2_000)))
        .and_then(|_| mgr.channels[1].send(Action::Open(Duration::from_millis(2_000))))
        .and_then(|_| mgr.channels[2].send(Action::Open(Duration::from_millis(2_000))))
        .and_then(|_| mgr.channels[3].send(Action::Open(Duration::from_millis(2_000))))
        .and_then(|_| mgr.channels[4].send(Action::Open(Duration::from_millis(2_000))))
        .map_err(|_| error::ErrorInternalServerError("Message sending failed."))
        .map(|_| "Demo started.")
}
