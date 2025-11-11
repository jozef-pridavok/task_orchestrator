use anyhow::Result;
use std::env;
use task_orchestrator::{
    orchestrator::TaskOrchestrator,
    task::{read_tasks_from_csv, write_results_to_csv},
};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        anyhow::bail!("Usage: {} <tasks.csv>", args[0]);
    }

    let csv_file_path = &args[1];

    let tasks = read_tasks_from_csv(csv_file_path).await?;

    let orchestrator = TaskOrchestrator::new();
    let results = if tasks.len() > 1000 {
        // Use streaming variant for large number of tasks
        orchestrator.execute_tasks_streaming(tasks).await
    } else {
        orchestrator.execute_tasks(tasks).await
    };

    let output = write_results_to_csv(&results)?;
    print!("{output}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_integration_flow() {
        let csv_content = "task_id,task_type\n101,process_data\n102,process_data\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let tasks = read_tasks_from_csv(file_path).await.unwrap();
        assert_eq!(tasks.len(), 2);

        let orchestrator = TaskOrchestrator::new();
        let results = orchestrator.execute_tasks(tasks).await;
        assert_eq!(results.len(), 2);

        let output = write_results_to_csv(&results).unwrap();
        assert!(output.contains("task_id,final_status,error_info"));
        assert!(output.contains("101,"));
        assert!(output.contains("102,"));
    }
}
