mod client;
mod config;
mod conversation;

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use client::ChatClient;
use config::Config;
use conversation::Conversation;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;
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
