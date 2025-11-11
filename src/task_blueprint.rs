use anyhow::Result;
use tokio::time::{sleep, Duration};

pub struct TaskBlueprint;

impl TaskBlueprint {
    pub async fn execute(task_id: u64) -> Result<()> {
        Self::fetch_data("https://httpbin.org/get").await?;
        Self::long_delay().await;
        Self::emit_event(task_id).await;
        Ok(())
    }

    async fn fetch_data(url: &str) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        // Consume the response body to ensure the request is complete
        let _body = response.text().await?;

        Ok(())
    }

    async fn long_delay() {
        sleep(Duration::from_secs(5)).await;
    }

    async fn emit_event(task_id: u64) {
        eprintln!("Task {task_id} completed successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_data_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("ok")
            .create_async()
            .await;

        let url = server.url() + "/test";
        let result = TaskBlueprint::fetch_data(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_data_failure() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(500)
            .create_async()
            .await;

        let url = server.url() + "/test";
        let result = TaskBlueprint::fetch_data(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
    }
}
