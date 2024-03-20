use anyhow::anyhow;
use aws_sdk_lambda as lambda;
use aws_sdk_lambda::operation::get_function_configuration::GetFunctionConfigurationOutput;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::{Environment, LastUpdateStatus, State};
use log::{info, warn};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

pub struct LambdaInvoker {
    lambda_client: lambda::Client,
    function_name: String,
    payload: Blob,
}

impl LambdaInvoker {
    pub fn new(lambda_client: lambda::Client, function_name: String, payload: String) -> Self {
        Self {
            lambda_client,
            function_name,
            payload: Blob::new(payload.into_bytes()),
        }
    }

    pub async fn iterate(&self, iterations: u8) -> Result<(), anyhow::Error> {
        let config = self.get_function_configuration().await?;
        let mut env = config
            .environment
            .map(|a| a.variables.unwrap_or_default())
            .unwrap_or_default();

        for i in 1..(iterations + 1) {
            info!("Iteration {i}/{iterations}");
            env.insert("cold_start_uuid".to_string(), Uuid::new_v4().to_string());
            self.refresh_lambda(env.clone()).await?;
            self.wait_for_function_ready().await?;
            self.invoke().await?;
        }
        info!("Done");
        Ok(())
    }

    async fn invoke(&self) -> Result<(), anyhow::Error> {
        info!("Invoking function");
        let result = self
            .lambda_client
            .invoke()
            .function_name(self.function_name.clone())
            .payload(self.payload.clone())
            .send()
            .await?;
        info!("Function error: {:?}", result.function_error);
        Ok(())
    }

    async fn get_function_configuration(
        &self,
    ) -> Result<GetFunctionConfigurationOutput, anyhow::Error> {
        info!("Getting function configuration");
        self.lambda_client
            .get_function_configuration()
            .function_name(self.function_name.to_string())
            .send()
            .await
            .map_err(anyhow::Error::from)
    }

    async fn refresh_lambda(&self, env: HashMap<String, String>) -> Result<(), anyhow::Error> {
        self.lambda_client
            .update_function_configuration()
            .function_name(self.function_name.to_string())
            .environment(
                Environment::builder()
                    .set_variables(Some(env.clone()))
                    .build(),
            )
            .send()
            .await
            .map(|_| ())
            .map_err(anyhow::Error::from)
    }

    async fn wait_for_function_ready(&self) -> Result<(), anyhow::Error> {
        info!("Waiting for function");
        while !self.is_function_ready().await? {
            info!("Function is not ready, sleeping 1s");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    async fn is_function_ready(&self) -> Result<bool, anyhow::Error> {
        let config = self.get_function_configuration().await?;
        if let Some(state) = config.state() {
            info!("Checking if function is active: {state}");
            if !matches!(state, State::Active) {
                return Ok(false);
            }
        }
        match config.last_update_status() {
            Some(last_update_status) => {
                info!("Checking if last update is successful: {last_update_status}");
                match last_update_status {
                    LastUpdateStatus::Successful => {
                        return Ok(true);
                    }
                    LastUpdateStatus::Failed | LastUpdateStatus::InProgress => {
                        return Ok(false);
                    }
                    unknown => {
                        warn!("LastUpdateStatus unknown: {unknown}");
                        return Err(anyhow!("Unknown LastUpdateStatus, fn config is {config:?}"));
                    }
                }
            }
            None => {
                warn!("Missing last update status");
                return Ok(false);
            }
        };
    }
}
