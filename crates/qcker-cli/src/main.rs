mod commands;
mod output;
mod tui;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use tui::app::App;
use tui::event::{AppEvent, EventHandler};
use tui::handler::handle_key_event;
use tui::ui::draw;

#[derive(Parser)]
#[command(name = "qcker", version, about = "Qcker - A daemonless, rootless container engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true, default_value = "text")]
    format: String,

    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Create(commands::create::CreateArgs),
    Start(commands::start::StartArgs),
    Kill(commands::kill::KillArgs),
    Delete(commands::delete::DeleteArgs),
    State(commands::state::StateArgs),
    Run(commands::run::RunArgs),
    Ps(commands::ps::PsArgs),
    Pull(commands::pull::PullArgs),
    Images(commands::images::ImagesArgs),
    Build(commands::build::BuildArgs),
    Network(commands::network::NetworkArgs),
    Volume(commands::volume::VolumeArgs),
    Compose(commands::compose::ComposeArgs),
    Extension(commands::extension::ExtensionArgs),
    Exec(commands::exec::ExecArgs),
    Logs(commands::logs::LogsArgs),
    Stop(commands::stop::StopArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("qcker")
    });

    match cli.command {
        Some(cmd) => {
            match cmd {
                Commands::Create(args) => commands::create::execute(args, &data_dir, &cli.format),
                Commands::Start(args) => commands::start::execute(args, &data_dir, &cli.format),
                Commands::Kill(args) => commands::kill::execute(args, &data_dir, &cli.format),
                Commands::Delete(args) => commands::delete::execute(args, &data_dir, &cli.format),
                Commands::State(args) => commands::state::execute(args, &data_dir, &cli.format),
                Commands::Run(args) => commands::run::execute(args, &data_dir, &cli.format),
                Commands::Ps(args) => commands::ps::execute(args, &data_dir, &cli.format),
                Commands::Pull(args) => commands::pull::execute(args, &data_dir, &cli.format).await,
                Commands::Images(args) => commands::images::execute(args, &data_dir, &cli.format),
                Commands::Build(args) => commands::build::execute(args, &data_dir, &cli.format),
                Commands::Network(args) => commands::network::execute(args, &data_dir, &cli.format),
                Commands::Volume(args) => commands::volume::execute(args, &data_dir, &cli.format),
                Commands::Compose(args) => commands::compose::execute(args, &data_dir, &cli.format),
                Commands::Extension(args) => commands::extension::execute(args, &data_dir, &cli.format),
                Commands::Exec(args) => commands::exec::execute(args, &data_dir, &cli.format),
                Commands::Logs(args) => commands::logs::execute(args, &data_dir, &cli.format),
                Commands::Stop(args) => commands::stop::execute(args, &data_dir, &cli.format),
            }
        }
        None => {
            run_tui(data_dir)
        }
    }
}

fn run_tui(data_dir: PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let events = EventHandler::new(tick_rate);
    let mut app = App::new(data_dir);
    app.refresh();

    loop {
        terminal.draw(|f| draw(f, &app))?;

        match events.next()? {
            AppEvent::Input(key) => {
                handle_key_event(&mut app, key);
            }
            AppEvent::Tick => {}
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
