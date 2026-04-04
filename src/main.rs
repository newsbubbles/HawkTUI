//! HawkTUI - A premium TUI wrapper for pi_agent_rust
//!
//! 🦅 "See everything. Control everything. Code like a hawk."

use clap::Parser;
use hawktui::App;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// HawkTUI - AI Coding Agent Terminal Interface
#[derive(Parser, Debug)]
#[command(name = "hawk")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Initial message to send to the agent
    #[arg(value_name = "MESSAGE")]
    message: Option<String>,

    /// Continue the last session
    #[arg(short, long)]
    r#continue: bool,

    /// Resume a specific session by name or ID
    #[arg(short = 's', long, value_name = "SESSION")]
    session: Option<String>,

    /// Model to use (e.g., claude-sonnet-4-20250514, gpt-4o)
    #[arg(short, long, value_name = "MODEL")]
    model: Option<String>,

    /// Provider to use (e.g., anthropic, openai)
    #[arg(short, long, value_name = "PROVIDER")]
    provider: Option<String>,

    /// Theme to use (hawk-dark, hawk-light, cyberpunk)
    #[arg(short, long, value_name = "THEME", default_value = "hawk-dark")]
    theme: String,

    /// Layout mode (command-center, focus, split)
    #[arg(short, long, value_name = "LAYOUT", default_value = "command-center")]
    layout: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Print mode - single response, no TUI
    #[arg(long)]
    print: bool,

    /// List available models
    #[arg(long)]
    list_models: bool,

    /// List available providers
    #[arg(long)]
    list_providers: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("hawktui=debug,pi=debug")
    } else {
        EnvFilter::new("hawktui=info,pi=warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    // Handle info commands
    if cli.list_models {
        println!("\n\x1b[1;36m🦅 Available Models\x1b[0m\n");
        println!("  \x1b[33mAnthropic:\x1b[0m");
        println!("    • claude-sonnet-4-20250514 (default)");
        println!("    • claude-opus-4-20250514");
        println!("    • claude-3-5-haiku-20241022");
        println!("\n  \x1b[33mOpenAI:\x1b[0m");
        println!("    • gpt-4o");
        println!("    • gpt-4-turbo");
        println!("    • o1-preview");
        println!();
        return Ok(());
    }

    if cli.list_providers {
        println!("\n\x1b[1;36m🦅 Available Providers\x1b[0m\n");
        println!("  • anthropic (default)");
        println!("  • openai");
        println!("  • openrouter");
        println!("  • bedrock");
        println!("  • vertex");
        println!();
        return Ok(());
    }

    // Print mode - single shot without TUI
    if cli.print {
        if let Some(msg) = &cli.message {
            use hawktui::providers::PiBridge;
            use hawktui::core::error::Result;
            
            println!("\n\x1b[2m[Print mode - sending message...]\x1b[0m\n");
            println!("\x1b[1;32mYou:\x1b[0m {msg}");
            println!("\n\x1b[1;36m🤖 Assistant:\x1b[0m");
            
            async fn run_print_mode(msg: &str, model: Option<String>, provider: Option<String>) -> Result<String> {
                let mut bridge = PiBridge::new(model, provider);
                bridge.connect().await?;
                
                let response = bridge.send_message(msg, |event| {
                    // In print mode, we could print chunks as they arrive
                    tracing::debug!(?event, "Agent event");
                }).await?;
                
                // Extract text from response
                let text = response
                    .content
                    .iter()
                    .filter_map(|block| match block {
                        pi::model::ContentBlock::Text(tc) => Some(tc.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                
                Ok(text)
            }
            
            match run_print_mode(msg, cli.model, cli.provider).await {
                Ok(response) => println!("{response}\n"),
                Err(e) => eprintln!("\x1b[31mError: {e}\x1b[0m\n"),
            }
        } else {
            eprintln!("\x1b[31mError: Print mode requires a message\x1b[0m");
            std::process::exit(1);
        }
        return Ok(());
    }

    // Create and run the TUI app
    let mut app = App::builder()
        .theme(&cli.theme)
        .layout(&cli.layout)
        .session(cli.session)
        .model(cli.model)
        .provider(cli.provider)
        .continue_last(cli.r#continue)
        .initial_message(cli.message)
        .build()?;

    Ok(app.run().await?)
}
