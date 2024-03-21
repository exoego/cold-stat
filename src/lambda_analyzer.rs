use crate::stats::Stats;
use async_recursion::async_recursion;
use aws_sdk_cloudwatchlogs as cloudwatch_logs;
use aws_sdk_cloudwatchlogs::types::QueryStatus;
use log::info;
use std::time::Duration;

const QUERY_STRING: &str = "
    fields @memorySize / 1000000 as memorySize
    | filter @message like /(?i)(Init Duration)/
    | stats count() as count,
    min(@initDuration ) as min,
    max(@initDuration ) as max,
    stddev(@initDuration ) as stddev,
    pct(@initDuration, 25) as p50,
    pct(@initDuration, 75) as p75,
    pct(@initDuration, 99) as p99,
    pct(@initDuration, 99.5) as p995,
    pct(@initDuration, 99.9) as p999
    by memorySize";

pub struct LambdaAnalyzer {
    cloudwatch_logs_client: cloudwatch_logs::Client,
    log_group_name: String,
    start_time: i64,
}

impl LambdaAnalyzer {
    pub fn new(
        cloudwatch_logs_client: cloudwatch_logs::Client,
        log_group_name: String,
        start_time: i64,
    ) -> Self {
        Self {
            cloudwatch_logs_client,
            log_group_name,
            start_time,
        }
    }

    pub async fn analyze(&self) -> Result<Stats, anyhow::Error> {
        info!(
            "Analyzing logs in log group: {}",
            self.log_group_name.to_string()
        );
        self.check_log_group().await?;

        let query_id = self
            .cloudwatch_logs_client
            .start_query()
            .query_string(QUERY_STRING)
            .log_group_name(self.log_group_name.to_string())
            .start_time(self.start_time)
            .end_time(chrono::Utc::now().timestamp())
            .send()
            .await?
            .query_id
            .unwrap();
        self.query_until_complete(query_id).await
    }

    async fn check_log_group(&self) -> Result<(), anyhow::Error> {
        let log_group = self
            .cloudwatch_logs_client
            .describe_log_groups()
            .log_group_name_pattern(self.log_group_name.to_string())
            .send()
            .await?;
        if log_group.log_groups.unwrap_or_else(|| vec![]).is_empty() {
            return Err(anyhow::anyhow!(
                "Log group {} does not exist",
                self.log_group_name.to_string()
            ));
        }
        Ok(())
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
                    let mut stats = Stats::default();
                    query_results
                        .results
                        .clone()
                        .unwrap()
                        .iter()
                        .flatten()
                        .for_each(|result| {
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
