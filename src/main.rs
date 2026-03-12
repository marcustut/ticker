use std::{path::PathBuf, process::Command, sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use chrono::Utc;
use clap::Parser;
use sentinel::StorageTyped;
use tracing_subscriber::{EnvFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, Job};

pub mod config;

type ThreadId = String;

#[derive(Clone)]
struct ThreadPoolState {
    config: Arc<ArcSwap<Config>>,
}

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(short, long)]
    config_file: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    // Setup logging to console and file
    let file_appender = tracing_appender::rolling::daily("./logs", "ticker.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(
            Layer::new()
                .with_thread_names(true)
                .with_writer(std::io::stdout),
        )
        .with(
            Layer::new()
                .with_thread_names(true)
                .with_writer(non_blocking_writer),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let watcher = sentinel::file::FileWatcher::new(args.config_file)?;

        let config: Config = watcher.load().await?;
        let state = ThreadPoolState {
            config: ArcSwap::from_pointee(config).into(),
        };

        let mut pool = threadpool::ThreadPool::new(state.clone());

        // Closure for restarting all jobs
        let mut restart = || {
            pool.ids()
                .iter()
                .for_each(|id: &String| pool.stop(id.clone()).expect("failed to stop job"));
            state.config.load().jobs.iter().for_each(|(name, job)| {
                pool.spawn(name.clone(), job_handler(job.clone()))
                    .expect("failed to spawn job");
                tracing::info!("Spawned job {name}: {job:?}");
            });
        };

        restart();
        while let Some(config) = sentinel::StorageTyped::<Config>::watch(&watcher).await {
            tracing::info!(
                "Detected config change, stop {} jobs and start {} jobs",
                state.config.load().jobs.len(),
                config.jobs.len()
            );
            state.config.store(config.into());
            restart();
        }

        Ok::<_, color_eyre::eyre::Error>(())
    })
}

fn job_handler(
    job: Job,
) -> Arc<
    dyn Fn(ThreadId, ThreadPoolState, std::sync::mpsc::Receiver<()>) -> std::thread::JoinHandle<()>
        + Send
        + Sync,
> {
    Arc::new(move |name, state, shutdown_rx| {
        let job = job.clone();

        std::thread::spawn(move || {
            let now = || Utc::now().with_timezone(&state.config.load().timezone);
            let sleep = || std::thread::sleep(Duration::from_millis(100));

            let mut next_resync_time = job.trigger.find_next_occurrence(&now(), false).unwrap();

            while let Err(std::sync::mpsc::TryRecvError::Empty) = shutdown_rx.try_recv() {
                if now() < next_resync_time {
                    sleep();
                    continue;
                }

                tracing::info!("[{name}] Running `{}`", job.command);
                run_shell_command(&job.command);

                next_resync_time = job.trigger.find_next_occurrence(&now(), false).unwrap();
            }
        })
    })
}

fn run_shell_command(command_string: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command_string])
            .output()
            .expect("failed to execute process on Windows")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command_string)
            .output()
            .expect("failed to execute process on Unix")
    };

    // Convert the stdout bytes to a UTF-8 string
    str::from_utf8(&output.stdout)
        .expect("failed to convert stdout to string")
        .to_string()
}
