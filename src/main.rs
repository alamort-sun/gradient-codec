use clap::Parser;

mod adapter;
mod benchmark;
mod boundary;
mod plane;
mod budget;
mod cli;
mod critique;
mod errors;
mod routing;
mod trace;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gradient_codec=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            prompt,
            budget,
            policy,
            provider,
        } => {
            cli::run_command(prompt, budget, policy, provider).await?;
        }
        Commands::Benchmark { policies, tasks } => {
            cli::benchmark_command(policies, tasks).await?;
        }
        Commands::Policies => {
            cli::list_policies_command()?;
        }
    }

    Ok(())
}
