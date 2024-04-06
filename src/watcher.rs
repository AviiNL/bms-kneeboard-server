use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, Receiver},
    },
    time::sleep,
};

use crate::Options;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = mpsc::channel(1);

    let watcher = recommended_watcher(move |res| {
        let tx = tx.clone();
        futures::executor::block_on(async {
            let _ = tx.send(res).await;
        });
    })?;

    Ok((watcher, rx))
}

pub fn start(options: Arc<Options>, tx: mpsc::Sender<()>, mut close_rx: broadcast::Receiver<()>) {
    let (mut watcher, mut rx) = async_watcher().unwrap();

    tokio::spawn(async move {
        let mut closed = false;
        let mut _briefing = None;
        loop {
            _briefing = options.briefing.read().unwrap().clone();
            let Some(briefing) = _briefing.as_ref() else {
                tokio::select! {
                    _ = sleep(Duration::from_millis(300)) => {
                        continue;
                    },
                    _ = close_rx.recv() => {
                        closed = true;
                        break;
                    }
                }
            };

            if watcher.watch(briefing, RecursiveMode::NonRecursive).is_ok() {
                // file exists
                println!("File loaded, poking");
                let _ = tx.send(()).await;
                break;
            }
            tokio::select! {
                _ = sleep(Duration::from_millis(300)) => {
                    // file doesnt exist (yet) try again in a bit
                },
                _ = close_rx.recv() => {
                    let _ = watcher.unwatch(briefing);
                    closed = true;
                    break;
                }
            }
        }

        if closed {
            return;
        }

        loop {
            tokio::select! {
                _ = rx.recv() => {
                    println!("File changed, poking");
                    let _ = tx.send(()).await;
                }
                _ = close_rx.recv() => {
                    if let Some(briefing) = _briefing {
                        let _ = watcher.unwatch(&briefing);
                    }
                    break;
                }
            }
        }
    });
}
