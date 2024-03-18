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
use notify::{recommended_watcher, RecursiveMode, Watcher};
use serde::Serialize;
use serde_type_name::type_name;
use std::{
    fs::File, io::Read, net::SocketAddr, path::PathBuf, result::Result, sync::OnceLock,
    thread::sleep, time::Duration,
};
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
    /// Override directory containing briefing.txt
    briefing_dir: Option<PathBuf>,
}

fn listen_address() -> SocketAddr {
    match "127.0.0.1:7878".parse() {
        Ok(p) => p,
        Err(e) => panic!("Invalid address: {:?}", e),
    }
}

static ARGS: OnceLock<Args> = OnceLock::new();
static BRIEFING_PATH: OnceLock<PathBuf> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let args = ARGS.get_or_init(move || args);
    let briefing_path = get_briefings_path(args.briefing_dir.as_ref());

    let sse_service = SseService::new();

    let app = Router::new()
        .route("/", get(index))
        .route("/sse", get(sse))
        .layer(Extension(sse_service.clone()));

    // run it
    let listener = tokio::net::TcpListener::bind(args.listen).await.unwrap();

    tokio::spawn(async move {
        println!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    });

    let sse_service_1 = sse_service.clone();
    let mut watcher = recommended_watcher(move |res| match res {
        Ok(_) => {
            let _ = sse_service_1.push(&Poke {});
        }
        Err(e) => println!("watch error: {:?}", e),
    })
    .unwrap();

    let mut briefing = args.briefing_dir.as_ref().unwrap_or(briefing_path).clone();
    briefing.push("briefing.txt");

    println!("Watching [{}] for changes", briefing.to_string_lossy());
    loop {
        if watcher
            .watch(briefing.as_path(), RecursiveMode::NonRecursive)
            .is_ok()
        {
            let _ = sse_service.push(&Poke {});
            break;
        }
        sleep(Duration::from_millis(300));
        // File doesn't exist (yet) wait for it.
    }

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
    let Some(briefing_path) = BRIEFING_PATH.get() else {
        return Html("404".to_string());
    };
    // User/Briefings/briefing.txt";
    let mut briefing = briefing_path.clone();
    briefing.push("briefing.txt");

    let briefing = match File::open(briefing) {
        Ok(e) => e,
        Err(e) => {
            println!("{:?}", e);
            return Html(
                Index {
                    package_elements: vec![],
                    steerpoints: vec![],
                    commladder: vec![],
                }
                .call()
                .unwrap(),
            );
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

fn get_briefings_path(bms_path: Option<&PathBuf>) -> &PathBuf {
    BRIEFING_PATH.get_or_init(move || {
        if let Some(bms_path) = bms_path {
            return bms_path.clone();
        }

        // there are more
        use bms_sm::*;

        println!("Waiting for Falcon BMS");

        let mut strings = StringData::read();
        loop {
            if strings.is_ok()
                && !strings.as_ref().unwrap()[&StringId::BmsBriefingsDirectory].is_empty()
            {
                break;
            }
            sleep(Duration::from_secs(1));
            strings = StringData::read();
        }
        let strings = strings.unwrap();

        strings[&StringId::BmsBriefingsDirectory].clone().into()
    })
}
