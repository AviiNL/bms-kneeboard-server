use std::{fs::File, io::Read, net::SocketAddr, sync::Arc};

use axum::{
    extract::Path,
    http::Response,
    response::{
        sse::{Event, KeepAlive},
        Html, Sse,
    },
    routing::get,
    Extension, Router,
};
use bms_briefing_parser::*;
use convert_case::{Case, Casing};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;
use serde_type_name::type_name;
use tera::Context;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    Stream,
};

use crate::{html, Options};

pub fn start(
    listen: SocketAddr,
    options: Arc<Options>,
    mut rx: mpsc::Receiver<()>,
    mut close_rx: broadcast::Receiver<()>,
    mut close_sse_rx: broadcast::Receiver<()>,
) {
    let sse_service = SseService::new();

    let sse_poker = sse_service.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    let _ = sse_poker.push(&Poke {});
                }
                _ = close_sse_rx.recv() => {
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        let app = Router::new()
            .route("/sse", get(sse))
            .route("/favicon.ico", get(a404))
            .route("/style.css", get(style))
            .route("/", get(index))
            .route("/*key", get(index_params))
            .layer(Extension(options.clone()))
            .layer(Extension(sse_service.clone()));

        let listener = tokio::net::TcpListener::bind(listen).await.unwrap();

        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                close_rx.recv().await.ok();
            })
            .await
        {
            eprintln!("Axum error: {:?}", e);
        }
    });
}

#[derive(Serialize)]
struct Poke;

async fn style() -> Response<String> {
    // read file next to executable called style.css
    let file = std::fs::read_to_string("style.css").unwrap_or_default(); // empty string as default

    Response::new(file)
}

async fn a404() -> Response<String> {
    Response::new("404".to_owned())
}

async fn sse(
    Extension(sse): Extension<SseService>,
) -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
    Sse::new(BroadcastStream::new(sse.subscribe())).keep_alive(KeepAlive::default())
}

async fn index(Extension(options): Extension<Arc<Options>>) -> Html<String> {
    index_params(axum::Extension(options), Path("PESPCL".to_string())).await
}

async fn index_params(
    Extension(options): Extension<Arc<Options>>,
    Path(key): Path<String>,
) -> Html<String> {
    let Ok(subs) = key
        .as_bytes()
        .chunks(2)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
    else {
        return Html("501".to_string());
    };

    let briefing = &options.briefing.read().unwrap();

    let mut context = Context::new();

    context.insert("subs", &subs);

    let Some(briefing) = briefing.as_ref() else {
        context.insert("msg", "Waiting for Falcon BMS to launch...");

        let render = match html::render(context) {
            Ok(e) => e,
            Err(e) => {
                println!("{:?}", e);
                return Html(String::from("501"));
            }
        };

        return Html(render);
    };

    let briefing = match File::open(briefing) {
        Ok(e) => e,
        Err(e) => {
            dbg!(e);
            context.insert("msg", "Waiting for briefing to be printed...");

            let render = match html::render(context) {
                Ok(e) => e,
                Err(e) => {
                    println!("{:?}", e);
                    return Html(String::from("501"));
                }
            };

            return Html(render);
        }
    };

    let mut buf = String::new();
    if let Err(e) = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(briefing)
        .read_to_string(&mut buf)
    {
        println!("{:?}", e);
        return Html(String::from("501"));
    }
    buf = buf
        .replace("\r\n", "\n")
        .replace(|c: char| !c.is_ascii(), "");

    let mut commladder = Comm::from_briefing(&buf);

    commladder.iter_mut().for_each(|c| {
        if let Some(callsign) = c.callsign.as_mut() {
            if let Some(c) = callsign
                .split(|c: char| !c.is_alphanumeric() && !c.is_whitespace() && c != '-')
                .next()
            {
                *callsign = c;
            }
        };
    });

    let overview = Overview::from_briefing(&buf);
    let sitrep = Sitrep::from_briefing(&buf);
    let roster = PilotRoster::from_briefing(&buf);
    let elements = PackageElement::from_briefing(&buf);
    let threatanalysis = ThreatAnalysis::from_briefing(&buf);
    let steerpoints = Steerpoint::from_briefing(&buf);
    let iff = Iff::from_briefing(&buf);
    let ordnance = Ordnance::from_briefing(&buf);
    let weather = Weather::from_briefing(&buf);
    let support = Support::from_briefing(&buf);
    let roe = RulesOfEngagement::from_briefing(&buf);
    let emergency = Emergency::from_briefing(&buf);

    context.insert("overview", &overview);
    context.insert("sitrep", &sitrep);
    context.insert("roster", &roster);
    context.insert("elements", &elements);
    context.insert("threatanalysis", &threatanalysis);
    context.insert("steerpoints", &steerpoints);
    context.insert("commladder", &commladder);
    context.insert("iff", &iff);
    context.insert("ordnance", &ordnance);
    context.insert("weather", &weather);
    context.insert("support", &support);
    context.insert("roe", &roe);
    context.insert("emergency", &emergency);

    let render = match html::render(context) {
        Ok(e) => e,
        Err(e) => {
            println!("{:?}", e);
            return Html(String::from("501"));
        }
    };

    Html(render)
}

#[derive(Debug, Clone)]
pub struct SseService {
    tx: broadcast::Sender<Event>,
}

impl Default for SseService {
    fn default() -> Self {
        Self {
            tx: broadcast::channel(100).0,
        }
    }
}

impl SseService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    #[allow(unused)]
    pub fn push<T>(&self, data: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: ?Sized + Serialize,
    {
        let name = type_name(&data)?.to_case(Case::Snake);
        let json = serde_json::to_string(data)?;

        let event = Event::default().event(name).data(json);
        self.tx.send(event)?;
        Ok(())
    }
}
