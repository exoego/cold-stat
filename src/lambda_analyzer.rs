use std::time::Duration;
use crate::stats::Stats;
use async_recursion::async_recursion;
use aws_sdk_cloudwatchlogs as cloudwatch_logs;
use aws_sdk_cloudwatchlogs::types::QueryStatus;
use log::{debug, info};

const QUERY_STRING: &str = "
    fields @memorySize / 1000000 as memorySize
    | filter @message like /(?i)(Init Duration)/
    | stats count() as cold_starts,
    min(@initDuration ) as min,
    max(@initDuration ) as max,
    pct(@initDuration, 25) as p50,
    pct(@initDuration, 75) as p75,
    pct(@initDuration, 99) as p99,
    pct(@initDuration, 99.5) as p995,
    pct(@initDuration, 99.9) as p999
    by memorySize";

pub struct LambdaAnalyzer {
    cloudwatch_logs_client: cloudwatch_logs::Client,
    function_name: String,
    start_time: i64,
}

impl LambdaAnalyzer {
    pub fn new(
        cloudwatch_logs_client: cloudwatch_logs::Client,
        function_name: String,
        start_time: i64,
    ) -> Self {
        Self {
            cloudwatch_logs_client,
            function_name,
            start_time,
        }
    }


    pub async fn analyze(&self) -> Result<Stats, anyhow::Error> {
        let log_group_name =
            "/aws/lambda/logs-export-development-aws".to_string();
        // format!("/aws/lambda/{}", self.function_name);
        info!("Analyzing logs in log group: {}", log_group_name);
        let query_id = self
            .cloudwatch_logs_client
            .start_query()
            .query_string(QUERY_STRING)
            .log_group_name(log_group_name)
            .start_time(self.start_time)
            .end_time(chrono::Utc::now().timestamp())
            .send()
            .await?
            .query_id
            .unwrap();
        self.query_until_complete(query_id).await
    }

    #[async_recursion]
    async fn query_until_complete(&self, query_id: String) -> Result<Stats, anyhow::Error> {
        info!("Fetching query result");
        let query_results = self
            .cloudwatch_logs_client
            .get_query_results()
            .query_id(query_id.to_string())
            .send()
            .await?;
        match query_results.status {
            None => {
                info!("Query is not complete, sleeping 1s");
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.query_until_complete(query_id.to_string()).await
            }
            Some(status) => match status {
                QueryStatus::Complete => {
                    info!("Query is complete, parsing results");
                    let mut stats = Stats::empty();
                    query_results
                        .results.clone()
                        .unwrap()
                        .iter()
                        .flatten().for_each(|result| {
                        stats.update(result);
                    });
                    Ok(stats.clone())
                }
                _ => {
                    info!("Query is not complete, sleeping 1s");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    self.query_until_complete(query_id).await
                }
            },
        }
    }
}
