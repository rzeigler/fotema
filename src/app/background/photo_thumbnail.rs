// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::*;
use futures::executor::block_on;
use rayon::prelude::*;
use relm4::prelude::*;
use relm4::Reducer;
use relm4::Worker;
use std::result::Result::Ok;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info};

use std::panic;

use crate::app::components::progress_monitor::{
    MediaType, ProgressMonitor, ProgressMonitorInput, TaskName,
};

#[derive(Debug)]
pub enum PhotoThumbnailInput {
    Start,
}

#[derive(Debug)]
pub enum PhotoThumbnailOutput {
    // Thumbnail generation has started.
    Started,

    // Thumbnail generation has completed
    Completed(usize),
}

pub struct PhotoThumbnail {
    // Stop flag
    stop: Arc<AtomicBool>,

    thumbnailer: fotema_core::photo::Thumbnailer,

    // Danger! Don't hold the repo mutex for too long as it blocks viewing images.
    repo: fotema_core::photo::Repository,

    progress_monitor: Arc<Reducer<ProgressMonitor>>,
}

impl PhotoThumbnail {
    fn enrich(
        stop: Arc<AtomicBool>,
        repo: fotema_core::photo::Repository,
        thumbnailer: fotema_core::photo::Thumbnailer,
        progress_monitor: Arc<Reducer<ProgressMonitor>>,
        sender: ComponentSender<Self>,
    ) -> Result<()> {
        let start = std::time::Instant::now();

        let mut unprocessed: Vec<fotema_core::photo::model::Picture> = repo
            .all()?
            .into_iter()
            .filter(|pic| pic.path.exists())
            .filter(|pic| !pic.thumbnail_path.as_ref().is_some_and(|p| p.exists()))
            .collect();

        // should be ascending time order from database, so reverse to process newest items first
        unprocessed.reverse();

        let count = unprocessed.len();
        info!("Found {} photos to generate thumbnails for", count);

        // Short-circuit before sending progress messages to stop
        // banner from appearing and disappearing.
        if count == 0 {
            let _ = sender.output(PhotoThumbnailOutput::Completed(count));
            return Ok(());
        }

        let _ = sender.output(PhotoThumbnailOutput::Started);

        progress_monitor.emit(ProgressMonitorInput::Start(
            TaskName::Thumbnail(MediaType::Photo),
            count,
        ));

        // One thread per CPU core... makes my laptop sluggish and hot... also likes memory.
        // Might need to consider constraining number of CPUs to use less memory or to
        // keep the computer more response while thumbnail generation is going on.
        unprocessed
            .par_iter()
            .take_any_while(|_| !stop.load(Ordering::Relaxed))
            .for_each(|pic| {
                // Careful! panic::catch_unwind returns Ok(Err) if the evaluated expression returns
                // an error but doesn't panic.
                let result = panic::catch_unwind(|| {
                    block_on(async { thumbnailer.thumbnail(&pic.picture_id, &pic.path).await })
                        .and_then(|thumbnail_path| {
                            repo.clone().add_thumbnail(&pic.picture_id, &thumbnail_path)
                        })
                });

                // If we got an err, then there was a panic.
                // If we got Ok(Err(e)) there wasn't a panic, but we still failed.
                if let Ok(Err(e)) = result {
                    error!(
                        "Failed generate or add thumbnail: {:?}: Photo path: {:?}",
                        e, pic.path
                    );
                    let _ = repo.clone().mark_broken(&pic.picture_id);
                } else if result.is_err() {
                    error!(
                        "Panicked generate or add thumbnail: Photo path: {:?}",
                        pic.path
                    );
                    let _ = repo.clone().mark_broken(&pic.picture_id);
                }

                progress_monitor.emit(ProgressMonitorInput::Advance);
            });

        info!(
            "Generated {} photo thumbnails in {} seconds.",
            count,
            start.elapsed().as_secs()
        );

        progress_monitor.emit(ProgressMonitorInput::Complete);

        let _ = sender.output(PhotoThumbnailOutput::Completed(count));

        Ok(())
    }
}

impl Worker for PhotoThumbnail {
    type Init = (
        Arc<AtomicBool>,
        fotema_core::photo::Thumbnailer,
        fotema_core::photo::Repository,
        Arc<Reducer<ProgressMonitor>>,
    );
    type Input = PhotoThumbnailInput;
    type Output = PhotoThumbnailOutput;

    fn init(
        (stop, thumbnailer, repo, progress_monitor): Self::Init,
        _sender: ComponentSender<Self>,
    ) -> Self {
        PhotoThumbnail {
            stop,
            thumbnailer,
            repo,
            progress_monitor,
        }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PhotoThumbnailInput::Start => {
                info!("Generating photo thumbnails...");
                let stop = self.stop.clone();
                let repo = self.repo.clone();
                let thumbnailer = self.thumbnailer.clone();
                let progress_monitor = self.progress_monitor.clone();

                // Avoid runtime panic from calling block_on
                rayon::spawn(move || {
                    if let Err(e) =
                        PhotoThumbnail::enrich(stop, repo, thumbnailer, progress_monitor, sender)
                    {
                        error!("Failed to update previews: {}", e);
                    }
                });
            }
        };
    }
}
