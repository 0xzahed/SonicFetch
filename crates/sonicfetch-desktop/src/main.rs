use anyhow::{Context, Result, bail};
use sonicfetch_core::DownloadState;
use sonicfetch_engine::DownloadManager;
use sonicfetch_storage::SqliteDownloadRepository;
use std::{path::PathBuf, str::FromStr};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).with_target(false).init();
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    if command == "help" || command == "--help" || command == "-h" { usage(); return Ok(()); }
    let data = data_dir()?;
    let repo = SqliteDownloadRepository::open(data.join("sonicfetch.sqlite3")).await?;
    let manager = DownloadManager::new(repo)?;
    match command.as_str() {
        "add" => {
            let url = args.next().context("missing URL")?;
            let dir = args.next().map(PathBuf::from).unwrap_or(std::env::current_dir()?);
            let mut events = manager.subscribe();
            let id = manager.add(&url, &dir).await?;
            println!("Added {id} to {}", dir.display());
            wait_for_terminal(&manager, &mut events, id).await?;
        }
        "list" => {
            let records=manager.list().await?;
            if records.is_empty(){println!("No downloads yet.");}
            for d in records { let size=d.total_bytes.map(|n|format!("{} / {}",format_bytes(d.downloaded_bytes.max(0) as u64),format_bytes(n.max(0) as u64))).unwrap_or_else(||format!("{}",format_bytes(d.downloaded_bytes.max(0) as u64))); println!("{}  {:?}  {:>18}  {}",d.id,d.state,size,d.final_path); if let Some(e)=d.error_message { println!("    {e}"); } }
        }
        "pause" | "resume" | "cancel" | "retry" | "remove" => {
            let id=Uuid::from_str(&args.next().context("missing download ID")?).context("invalid download ID")?;
            match command.as_str(){"pause"=>{manager.pause(id).await?;println!("Paused {id}")},"resume"=>{let mut rx=manager.subscribe();manager.resume(id).await?;println!("Resumed {id}");wait_for_terminal(&manager,&mut rx,id).await?},"cancel"=>{manager.cancel(id).await?;println!("Cancelled {id}")},"retry"=>{let mut rx=manager.subscribe();manager.retry(id).await?;println!("Retrying {id}");wait_for_terminal(&manager,&mut rx,id).await?},"remove"=>{let delete=args.any(|a|a=="--delete-file");if manager.remove(id,delete).await?{println!("Removed {id}")}else{bail!("download not found")}},_=>{}}
        }
        _ => { usage(); bail!("unknown command: {command}"); }
    }
    Ok(())
}

async fn wait_for_terminal(manager: &DownloadManager, events: &mut tokio::sync::broadcast::Receiver<sonicfetch_engine::manager::DownloadEvent>, id: Uuid) -> Result<()> {
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    loop { let event=tokio::select! { event=events.recv()=>event?, _=&mut interrupt=>{manager.pause(id).await?;eprintln!("\nPaused {id}. Resume with `sonicfetch resume {id}`.");return Ok(())} }; if event.id!=id {continue;} match event.state {
        DownloadState::Downloading => if let Some(bytes)=event.bytes { if let Some(total)=event.total { eprint!("\r{} / {}",format_bytes(bytes),format_bytes(total)); } else { eprint!("\r{}",format_bytes(bytes)); } },
        DownloadState::Completed=>{eprintln!("\nDownload complete.");return Ok(())},
        DownloadState::Failed=>{eprintln!("\nDownload failed. See `sonicfetch list` for details.");bail!("download failed")},
        DownloadState::Paused|DownloadState::Cancelled=>{eprintln!("\nDownload {:?}.",event.state);return Ok(())},_=>{}
    }}
}
fn data_dir()->Result<PathBuf>{
    if let Some(path)=std::env::var_os("XDG_DATA_HOME"){return Ok(PathBuf::from(path).join("sonicfetch"));}
    let home=std::env::var_os("HOME").context("HOME is not set")?;Ok(PathBuf::from(home).join(".local/share/sonicfetch"))
}
fn format_bytes(n:u64)->String{const U:[&str;5]=["B","KiB","MiB","GiB","TiB"];let mut v=n as f64;let mut i=0;while v>=1024.&&i<U.len()-1{v/=1024.;i+=1;}if i==0{format!("{n} B")}else{format!("{v:.1} {}",U[i])}}
fn usage(){println!("SonicFetch — secure local download manager\n\nUSAGE:\n  sonicfetch add <http(s)-url> [directory]\n  sonicfetch list\n  sonicfetch pause <id>\n  sonicfetch resume <id>\n  sonicfetch retry <id>\n  sonicfetch cancel <id>\n  sonicfetch remove <id> [--delete-file]\n\nDownloads are saved to the current directory by default. Queue data is kept in $XDG_DATA_HOME/sonicfetch.");}
