mod client;
mod config;
mod display;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kagi")]
#[command(about = "Kagi API CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query Kagi FastGPT (AI answer with sources)
    Search {
        /// Search query
        query: String,
        /// Show raw JSON response
        #[arg(long)]
        raw: bool,
        /// Disable response cache
        #[arg(long)]
        no_cache: bool,
    },
    /// Summarize a URL or text
    Summarize {
        /// URL to summarize (omit if using --text)
        url: Option<String>,
        /// Text to summarize instead of a URL
        #[arg(long)]
        text: Option<String>,
        /// Return bullet-point takeaways instead of a summary
        #[arg(long)]
        takeaway: bool,
        /// Target language for the summary (e.g. fr, de, es)
        #[arg(long)]
        lang: Option<String>,
    },
    /// Raw web search via Kagi Search API
    Web {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short = 'l', long)]
        limit: Option<u32>,
    },
    /// Set the Kagi API token
    Config {
        #[arg(long)]
        token: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { token } => set_config(token)?,
        Commands::Search { query, raw, no_cache } => {
            let cfg = config::load_config()?;
            let kagi = client::KagiClient::new(cfg.token);
            run_search(&kagi, &query, raw, no_cache).await?;
        }
        Commands::Summarize { url, text, takeaway, lang } => {
            let cfg = config::load_config()?;
            let kagi = client::KagiClient::new(cfg.token);
            run_summarize(&kagi, url.as_deref(), text.as_deref(), takeaway, lang.as_deref()).await?;
        }
        Commands::Web { query, limit } => {
            let cfg = config::load_config()?;
            let kagi = client::KagiClient::new(cfg.token);
            run_web_search(&kagi, &query, limit).await?;
        }
    }

    Ok(())
}

fn set_config(token: String) -> Result<()> {
    config::save_config(&config::Config { token })?;
    eprintln!("Configuration saved to {:?}", config::config_dir());
    Ok(())
}

async fn run_search(kagi: &client::KagiClient, query: &str, raw: bool, no_cache: bool) -> Result<()> {
    let cache = !no_cache;
    let resp = kagi.fastgpt(query, cache, true).await?;

    if raw {
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "output": resp.data.output,
            "tokens": resp.data.tokens,
            "references": resp.data.references.iter().map(|r| serde_json::json!({
                "title": r.title,
                "snippet": r.snippet,
                "url": r.url,
            })).collect::<Vec<_>>(),
        }))?;
        println!("{json}");
    } else {
        display::format_fastgpt(&resp.data);
        eprintln!("\n[{} tokens used]", resp.data.tokens);
    }

    Ok(())
}

async fn run_summarize(
    kagi: &client::KagiClient,
    url: Option<&str>,
    text: Option<&str>,
    takeaway: bool,
    lang: Option<&str>,
) -> Result<()> {
    if url.is_none() && text.is_none() {
        anyhow::bail!("Provide a URL argument or --text");
    }

    let summary_type = if takeaway { Some("takeaway") } else { Some("summary") };
    let resp = kagi.summarize(url, text, summary_type, lang).await?;

    display::format_summary(&resp.data);
    eprintln!("\n[{} tokens used]", resp.data.tokens);

    Ok(())
}

async fn run_web_search(kagi: &client::KagiClient, query: &str, limit: Option<u32>) -> Result<()> {
    let resp = kagi.search(query, limit).await?;

    if resp.data.is_empty() {
        eprintln!("No results.");
    } else {
        display::format_web_search(&resp.data);
        eprintln!("[{} results]", resp.data.len());
    }

    Ok(())
}
