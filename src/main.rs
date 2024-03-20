mod lambda_invoker;

use crate::lambda_invoker::LambdaInvoker;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::Client;
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    function: String,

    #[arg(short, long, default_value_t = 1)]
    iterations: u8,

    #[arg(short, long)]
    payload: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let Args {
        function,
        iterations,
        payload,
    } = Args::parse();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let lambda = Client::new(&config);
    let lambda_invoker = LambdaInvoker::new(lambda.clone(), function.clone(), payload);

    lambda_invoker.iterate(iterations).await?;
    Ok(())
}
