use crate::{
    task::{TaskInput, TaskResult, TaskStatus},
    task_blueprint::TaskBlueprint,
};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::mpsc;

pub struct TaskOrchestrator;

impl TaskOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_tasks(self, tasks: Vec<TaskInput>) -> Vec<TaskResult> {
        let (tx, mut rx) = mpsc::channel::<TaskResult>(1000);

        let mut handles = Vec::new();

        for task in tasks {
            let tx_clone = tx.clone();
            let handle = tokio::spawn(async move {
                let result = Self::execute_single_task(task.task_id).await;
                let _ = tx_clone.send(result).await;
            });
            handles.push(handle);
        }

        // Drop the original sender to signal completion
        drop(tx);

        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        for handle in handles {
            let _ = handle.await;
        }

        results
    }

    /// Streaming variant for handling large numbers of tasks efficiently (backpressure)
    pub async fn execute_tasks_streaming(self, tasks: Vec<TaskInput>) -> Vec<TaskResult> {
        let mut futures = FuturesUnordered::new();

        for task in tasks {
            futures.push(Self::execute_single_task(task.task_id));
        }

        let mut results = Vec::new();
        while let Some(result) = futures.next().await {
            results.push(result);
        }

        results
    }

    async fn execute_single_task(task_id: u64) -> TaskResult {
        let mut result = TaskResult {
            task_id,
            status: TaskStatus::Running,
            error_info: None,
        };

        match TaskBlueprint::execute(task_id).await {
            Ok(()) => result.status = TaskStatus::Completed,
            Err(e) => {
                result.status = TaskStatus::Failed;
                result.error_info = Some(e.to_string());
            }
        }

        result
    }
}

impl Default for TaskOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_multiple_tasks() {
        let orchestrator = TaskOrchestrator::new();
        let tasks = vec![
            TaskInput {
                task_id: 101,
                task_type: "process_data".to_string(),
            },
            TaskInput {
                task_id: 102,
                task_type: "process_data".to_string(),
            },
            TaskInput {
                task_id: 103,
                task_type: "process_data".to_string(),
            },
        ];

        let results = orchestrator.execute_tasks(tasks).await;
        assert_eq!(results.len(), 3);

        let task_ids: Vec<u64> = results.iter().map(|r| r.task_id).collect();
        assert!(task_ids.contains(&101));
        assert!(task_ids.contains(&102));
        assert!(task_ids.contains(&103));
    }

    #[tokio::test]
    async fn test_orchestrator_duplicate_task_ids() {
        let orchestrator = TaskOrchestrator::new();
        let tasks = vec![
            TaskInput {
                task_id: 101,
                task_type: "process_data".to_string(),
            },
            TaskInput {
                task_id: 101,
                task_type: "process_data".to_string(),
            },
        ];

        let results = orchestrator.execute_tasks(tasks).await;

        // Should have two results (deduplication happens in write_results_to_csv)
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.task_id == 101));
    }

    #[tokio::test]
    async fn test_orchestrator_streaming_variant() {
        let orchestrator = TaskOrchestrator::new();
        let tasks = vec![
            TaskInput {
                task_id: 201,
                task_type: "process_data".to_string(),
            },
            TaskInput {
                task_id: 202,
                task_type: "process_data".to_string(),
            },
        ];

        let results = orchestrator.execute_tasks_streaming(tasks).await;
        assert_eq!(results.len(), 2);

        let task_ids: Vec<u64> = results.iter().map(|r| r.task_id).collect();
        assert!(task_ids.contains(&201));
        assert!(task_ids.contains(&202));
    }
}
