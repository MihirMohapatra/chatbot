mod client;
mod config;
mod conversation;
mod web;

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use client::ChatClient;
use config::Config;
use conversation::Conversation;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("web");

    match mode {
        "web" | "ui" | "serve" => {
            let port: u16 = std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080);
            web::open_browser(port);
            web::start(port).await
        }
        _ => {
            let config = Config::load()?;
            run_cli(config).await
        }
    }
}

async fn run_cli(config: Config) -> Result<()> {
    let provider_label = config.provider.label().to_string();
    let model = config.model.clone();
    let client = ChatClient::new(config)?;
    let mut conv = Conversation::new();

    println!("╭──────────────────────────────────────────╮");
    println!("│  Rust Chatbot                            │");
    println!("│  Provider : {:<30}│", provider_label);
    println!("│  Model    : {:<30}│", model);
    println!("│  Type 'quit' or Ctrl-C to exit           │");
    println!("╰──────────────────────────────────────────╯");
    println!();

    let mut rl = DefaultEditor::new()?;

    loop {
        let line = rl.readline("You: ");

        match line {
            Ok(input) => {
                let input = input.trim().to_string();

                if input.is_empty() {
                    continue;
                }

                if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                rl.add_history_entry(&input)?;
                conv.add_user(&input);

                print!("Assistant: ");
                match client.send(&conv.messages).await {
                    Ok(reply) => {
                        println!("{}\n", reply);
                        conv.add_assistant(reply);
                    }
                    Err(e) => {
                        eprintln!("Error: {}\n", e);
                        conv.messages.pop();
                    }
                }
            }

            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("\nGoodbye!");
                break;
            }

            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
