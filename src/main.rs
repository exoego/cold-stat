mod lambda_analyzer;
mod lambda_invoker;
mod stats;

use crate::lambda_analyzer::LambdaAnalyzer;
use crate::lambda_invoker::LambdaInvoker;
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs as logs;
use aws_sdk_lambda as lambda;
use clap::Parser;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::iter;
use std::time::Duration;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::style::{BorderColor, LineChar, Offset};
use tabled::settings::themes::Colorization;
use tabled::settings::{Color, Modify, Style};
use tabled::Table;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Name or ARN of function to invoke")]
    function: String,

    #[arg(short, long, help = "JSON payload to send to the function")]
    payload: String,

    #[arg(long, default_value = None, help = "Name of CloudWatch log group to analyze.\n[default: /aws/lambda/FUNCTION]")]
    log_group_name: Option<String>,

    #[arg(long, default_value = None, help = "Regex to filter CloudWatch log group stream. Useful when log group is shared by multiple functions")]
    log_stream_filter: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = 100,
        help = "Number of iterations to invoke the function. It is recommended to set 30 at least. Because the number of collected cold starts often is a bit shorter than the specified `ITERATIONS` due to eventual consistency of CloudWatch Logs."
    )]
    iterations: u16,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Print debug logs if enabled"
    )]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let Args {
        function,
        log_group_name,
        iterations,
        log_stream_filter,
        payload,
        verbose,
    } = Args::parse();

    if verbose {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let lambda = lambda::Client::new(&config);
    let lambda_invoker = LambdaInvoker::new(lambda.clone(), function.clone(), payload);

    let start_time = chrono::Utc::now().timestamp();
    lambda_invoker.iterate(iterations).await?;

    info!("Waiting 30 seconds log buffer to be flushed");
    tokio::time::sleep(Duration::from_secs(30)).await;

    let logs = logs::Client::new(&config);
    let log_group_name_ = log_group_name.unwrap_or_else(|| {
        let exact_name = function.split(':').last().unwrap();
        format!("/aws/lambda/{}", exact_name)
    });
    let lambda_analyzer = LambdaAnalyzer::new(logs, log_group_name_, log_stream_filter, start_time);
    let stats = lambda_analyzer.analyze().await?;

    println!();
    println!(
        "{}",
        Table::new(stats)
            .with(Style::markdown())
            .with(Modify::new(Columns::new(..)).with(LineChar::horizontal(':', Offset::End(0))),)
            .with(
                BorderColor::new()
                    .set_top(Color::FG_WHITE)
                    .set_left(Color::FG_WHITE)
                    .set_right(Color::FG_WHITE)
                    .set_bottom(Color::FG_WHITE)
                    .set_corner_bottom_left(Color::FG_WHITE)
                    .set_corner_bottom_right(Color::FG_WHITE)
                    .set_corner_top_left(Color::FG_WHITE)
                    .set_corner_top_right(Color::FG_WHITE)
            )
            .with(Colorization::columns(
                iter::repeat(Color::FG_BRIGHT_CYAN).take(9)
            ))
            .modify(Rows::first(), Color::FG_BRIGHT_WHITE)
    );
    Ok(())
}
