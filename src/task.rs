use anyhow::Result;
use serde::{Deserialize, Serialize};
//use std::collections::HashMap;
use ahash::AHashMap as HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct TaskInput {
    pub task_id: u64,
    pub task_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: u64,
    pub status: TaskStatus,
    pub error_info: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskOutput {
    pub task_id: u64,
    pub final_status: String,
    pub error_info: String,
}

impl From<TaskResult> for TaskOutput {
    fn from(result: TaskResult) -> Self {
        let final_status = match result.status {
            TaskStatus::Completed => "Completed".to_string(),
            TaskStatus::Failed => "Failed".to_string(),
            _ => "Failed".to_string(),
        };

        let error_info = result.error_info.unwrap_or_default();

        TaskOutput {
            task_id: result.task_id,
            final_status,
            error_info,
        }
    }
}

pub async fn read_tasks_from_csv(file_path: &str) -> Result<Vec<TaskInput>> {
    let content = tokio::fs::read_to_string(file_path).await?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    let mut tasks = Vec::new();
    for result in reader.deserialize() {
        let task: TaskInput = result?;
        tasks.push(task);
    }

    Ok(tasks)
}

pub fn write_results_to_csv(results: &[TaskResult]) -> Result<String> {
    let mut unique_results: HashMap<u64, TaskResult> = HashMap::new();

    // Keep only the latest result for each task_id
    for result in results {
        unique_results.insert(result.task_id, result.clone());
    }

    let mut writer = csv::Writer::from_writer(vec![]);

    for result in unique_results.values() {
        let output: TaskOutput = result.clone().into();
        writer.serialize(output)?;
    }

    let data = writer.into_inner()?;
    Ok(String::from_utf8(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_tasks_from_csv() {
        let csv_content = "task_id,task_type\n101,process_data\n102,process_data\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let tasks = read_tasks_from_csv(file_path).await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].task_id, 101);
        assert_eq!(tasks[1].task_id, 102);
    }

    #[test]
    fn test_write_results_to_csv() {
        let results = vec![
            TaskResult {
                task_id: 101,
                status: TaskStatus::Completed,
                error_info: None,
            },
            TaskResult {
                task_id: 102,
                status: TaskStatus::Failed,
                error_info: Some("Network error".to_string()),
            },
        ];

        let output = write_results_to_csv(&results).unwrap();
        assert!(output.contains("task_id,final_status,error_info"));
        assert!(output.contains("101,Completed,"));
        assert!(output.contains("102,Failed,Network error"));
    }

    #[test]
    fn test_duplicate_task_ids() {
        let results = vec![
            TaskResult {
                task_id: 101,
                status: TaskStatus::Running,
                error_info: None,
            },
            TaskResult {
                task_id: 101,
                status: TaskStatus::Completed,
                error_info: None,
            },
        ];

        let output = write_results_to_csv(&results).unwrap();
        let lines: Vec<&str> = output.lines().filter(|line| !line.is_empty()).collect();
        assert_eq!(lines.len(), 2); // Header + 1 unique result
        assert!(output.contains("101,Completed,"));
    }
}
