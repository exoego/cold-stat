mod lambda_analyzer;
mod lambda_invoker;
mod stats;

use crate::lambda_analyzer::LambdaAnalyzer;
use crate::lambda_invoker::LambdaInvoker;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs as logs;
use aws_sdk_lambda as lambda;
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::iter;
use tabled::settings::object::Rows;
use tabled::settings::themes::Colorization;
use tabled::settings::{Color, Style};
use tabled::Table;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    function: String,

    #[arg(short, long, default_value_t = 1)]
    iterations: u8,

    #[arg(short, long)]
    payload: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let Args {
        function,
        iterations,
        payload,
        verbose,
    } = Args::parse();

    if verbose {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let lambda = lambda::Client::new(&config);
    let lambda_invoker = LambdaInvoker::new(lambda.clone(), function.clone(), payload);

    let start_time = chrono::Utc::now().timestamp() - 500000;
    lambda_invoker.iterate(iterations).await?;

    let logs = logs::Client::new(&config);
    let lambda_analyzer = LambdaAnalyzer::new(logs, function, start_time);
    let stats = lambda_analyzer.analyze().await?;

    println!(
        "{}",
        Table::new(vec![stats])
            .with(Style::blank())
            .with(Colorization::columns(
                iter::repeat(Color::FG_BRIGHT_CYAN).take(8)
            ))
            .modify(Rows::first(), Color::FG_WHITE)
    );
    Ok(())
}
