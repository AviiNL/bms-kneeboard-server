mod gameinfo;

use axum::{
    response::{
        sse::{Event, KeepAlive},
        Html, Sse,
    },
    routing::get,
    Extension, Router,
};
use bms_briefing_parser::*;
use clap::Parser;
use convert_case::{Case, Casing};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use gameinfo::GameInfo;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use serde::Serialize;
use serde_type_name::type_name;
use std::{fs::File, io::Read, net::SocketAddr, result::Result, sync::OnceLock};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    Stream,
};
use yarte::*;

#[derive(Serialize)]
struct Poke;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Webserver listen address:port
    #[arg(short, long, default_value_t = listen_address())]
    listen: SocketAddr,
}

fn listen_address() -> SocketAddr {
    match "127.0.0.1:7878".parse() {
        Ok(p) => p,
        Err(e) => panic!("Invalid address: {:?}", e),
    }
}

static ARGS: OnceLock<Args> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let args = ARGS.get_or_init(move || args);

    let Some(game_info) = GameInfo::new() else {
        panic!("Falcon BMS 4.37 not detected. Aborting!");
    };

    let sse_service = SseService::new();

    let app = Router::new()
        .route("/", get(index))
        .route("/sse", get(sse))
        .layer(Extension(sse_service.clone()));

    // run it
    let listener = tokio::net::TcpListener::bind(args.listen).await.unwrap();

    tokio::spawn(async move {
        println!("listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    });

    let mut watcher = recommended_watcher(move |res| match res {
        Ok(_) => sse_service.push(&Poke {}).unwrap(),
        Err(e) => println!("watch error: {:?}", e),
    })
    .unwrap();

    let mut briefing = game_info.base_dir.clone();
    briefing.push("User");
    briefing.push("Briefings");
    briefing.push("briefing.txt");

    println!("Watching [{}] for changes", briefing.to_string_lossy());
    watcher
        .watch(briefing.as_path(), RecursiveMode::NonRecursive)
        .unwrap();

    tokio::signal::ctrl_c().await.unwrap();
    println!("No longer watching...");
    Ok(())
}

#[derive(Template, Serialize)]
#[template(path = "index.hbs")]
pub struct Index {
    pub package_elements: Vec<PackageElement>,
    pub steerpoints: Vec<Steerpoint>,
    pub commladder: Vec<Comm>,
}

async fn index() -> Html<String> {
    let Some(game_info) = GameInfo::new() else {
        return Html("404".to_string());
    };
    // User/Briefings/briefing.txt";
    let mut briefing = game_info.base_dir.clone();
    briefing.push("User");
    briefing.push("Briefings");
    briefing.push("briefing.txt");

    let briefing = match File::open(briefing) {
        Ok(e) => e,
        Err(e) => {
            println!("{:?}", e);
            return Html(String::from("501"));
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

    Html(
        Index {
            package_elements: PackageElement::from_briefing(&buf),
            steerpoints: Steerpoint::from_briefing(&buf),
            commladder: Comm::from_briefing(&buf),
        }
        .call()
        .unwrap(),
    )
}

#[derive(Debug, Clone)]
pub struct SseService {
    tx: Sender<Event>,
}

impl Default for SseService {
    fn default() -> Self {
        Self { tx: channel(100).0 }
    }
}

impl SseService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> Receiver<Event> {
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

async fn sse(
    Extension(sse): Extension<SseService>,
) -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
    Sse::new(BroadcastStream::new(sse.subscribe())).keep_alive(KeepAlive::default())
}
