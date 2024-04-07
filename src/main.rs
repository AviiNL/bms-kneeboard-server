#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod html;
mod icon;
mod watcher;
mod web;

use bms_sm::{StringData, StringId};
use clap::Parser;
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Webserver listen address:port
    #[arg(short, long, default_value_t = listen_address())]
    listen: SocketAddr,
    /// Override directory containing briefing.txt, disabled autodetect
    briefing_dir: Option<PathBuf>,
}

fn listen_address() -> SocketAddr {
    match "127.0.0.1:7878".parse() {
        Ok(p) => p,
        Err(e) => panic!("Invalid address: {:?}", e),
    }
}

pub struct Options {
    pub briefing: RwLock<Option<PathBuf>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (tx, rx) = mpsc::channel::<()>(1);
    let (close_tx, close_rx) = broadcast::channel::<()>(1);

    let options = Arc::new(Options {
        briefing: RwLock::new(None),
    });

    let options_1 = options.clone();
    let tx_1 = tx.clone();

    let close_rx_1 = close_tx.subscribe();
    let briefing_dir = args.briefing_dir.clone();
    tokio::spawn(async move {
        let Some(briefing_path) = get_briefings_path(briefing_dir.as_ref(), close_rx_1).await
        else {
            return;
        };

        let mut briefing = args.briefing_dir.as_ref().unwrap_or(&briefing_path).clone();
        briefing.push("briefing.txt");

        *options_1.briefing.write().unwrap() = Some(briefing);
        let _ = tx_1.send(()).await;
    });

    web::start(
        args.listen,
        options.clone(),
        rx,
        close_rx,
        close_tx.subscribe(),
    );

    watcher::start(options.clone(), tx.clone(), close_tx.subscribe());

    icon::start(args.listen)?;

    close_tx.send(())?;

    Ok(())
}

async fn get_strings() -> HashMap<StringId, String> {
    loop {
        let strings = StringData::read();
        match strings {
            Ok(s) => {
                if !&s[&StringId::BmsBriefingsDirectory].is_empty() {
                    return s.clone();
                } else {
                    sleep(Duration::from_secs(1)).await;
                }
            }
            Err(_) => {
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn get_briefings_path(
    bms_path: Option<&PathBuf>,
    mut close_rx: broadcast::Receiver<()>,
) -> Option<PathBuf> {
    if let Some(bms_path) = bms_path {
        return Some(bms_path.clone());
    }

    use bms_sm::*;

    println!("Waiting for Falcon BMS");

    tokio::select! {
        strings = get_strings() => {
            Some(strings[&StringId::BmsBriefingsDirectory].clone().into())
        },
        _ = close_rx.recv() => {
            None
        }
    }
}
