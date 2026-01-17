use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};
use tracing::{info, debug, warn, error};

use super::{database::DatabaseInstance, error::AppError};

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task status for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
}

/// Serializable task data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub id: String,
    pub name: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

/// A task that can be queued and executed
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn priority(&self) -> TaskPriority;
    
    /// Serialize task for persistence
    fn to_data(&self) -> TaskData;
    
    /// Execute the task
    async fn execute(&self, db: Arc<DatabaseInstance>, client: reqwest::Client) -> Result<(), AppError>;
}

/// Wrapper for priority queue ordering
struct PriorityTask {
    task: Box<dyn Task>,
    priority: TaskPriority,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older tasks first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.created_at.cmp(&self.created_at),
            other => other,
        }
    }
}

/// Message types for the task queue
pub enum QueueMessage {
    /// Add a new task to the queue
    AddTask(Box<dyn Task>),
    /// Shutdown the queue
    Shutdown,
}

/// A task queue that executes tasks sequentially
pub struct TaskQueue {
    name: String,
    tx: mpsc::Sender<QueueMessage>,
}

impl TaskQueue {
    /// Create a new task queue
    /// Returns (TaskQueue, receiver handle for the worker)
    pub fn new(name: String, buffer_size: usize) -> (Self, mpsc::Receiver<QueueMessage>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (Self { name, tx }, rx)
    }

    /// Add a task to the queue
    pub async fn enqueue(&self, task: Box<dyn Task>) -> Result<(), AppError> {
        info!(
            queue = %self.name,
            task_id = %task.id(),
            task_name = %task.name(),
            priority = ?task.priority(),
            "Enqueueing task"
        );
        
        self.tx.send(QueueMessage::AddTask(task))
            .await
            .map_err(|e| AppError::Module(format!("Failed to enqueue task: {}", e)))
    }

    /// Shutdown the queue
    pub async fn shutdown(&self) -> Result<(), AppError> {
        info!(queue = %self.name, "Sending shutdown signal to queue");
        
        self.tx.send(QueueMessage::Shutdown)
            .await
            .map_err(|e| AppError::Module(format!("Failed to shutdown queue: {}", e)))
    }

    /// Get the queue name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Clone for TaskQueue {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            tx: self.tx.clone(),
        }
    }
}

/// Worker that processes tasks from the queue
pub struct QueueWorker {
    name: String,
    db: Arc<DatabaseInstance>,
    client: reqwest::Client,
}

impl QueueWorker {
    pub fn new(name: String, db: Arc<DatabaseInstance>, client: reqwest::Client) -> Self {
        Self { name, db, client }
    }

    /// Run the worker, processing tasks until shutdown
    pub async fn run(self, mut rx: mpsc::Receiver<QueueMessage>) -> Result<(), AppError> {
        info!(worker = %self.name, "Task queue worker started");
        
        let mut tasks_processed = 0;
        let mut priority_queue = BinaryHeap::new();
        
        // Load persisted tasks on startup
        if let Err(e) = self.load_persisted_tasks(&mut priority_queue).await {
            warn!(worker = %self.name, error = %e, "Failed to load persisted tasks");
        }

        loop {
            // Try to get next task from priority queue or wait for new one
            let task = if priority_queue.is_empty() {
                // Wait for new task
                match rx.recv().await {
                    Some(QueueMessage::AddTask(task)) => {
                        let priority = task.priority();
                        let created_at = chrono::Utc::now();
                        Some(PriorityTask { task, priority, created_at })
                    }
                    Some(QueueMessage::Shutdown) => {
                        info!(worker = %self.name, tasks_processed = tasks_processed, "Shutdown signal received");
                        break;
                    }
                    None => {
                        warn!(worker = %self.name, "Channel closed");
                        break;
                    }
                }
            } else {
                // Process highest priority task, but check for new messages
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(QueueMessage::AddTask(task)) => {
                                let priority = task.priority();
                                let created_at = chrono::Utc::now();
                                priority_queue.push(PriorityTask { task, priority, created_at });
                                None
                            }
                            Some(QueueMessage::Shutdown) => {
                                info!(worker = %self.name, "Shutdown during processing");
                                break;
                            }
                            None => break,
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
                        priority_queue.pop()
                    }
                }
            };
            
            if let Some(priority_task) = task {
                let task_id = priority_task.task.id();
                let task_name = priority_task.task.name();
                let priority = priority_task.priority;
                
                debug!(
                    worker = %self.name,
                    task_id = %task_id,
                    task_name = %task_name,
                    priority = ?priority,
                    queue_size = priority_queue.len(),
                    "Processing task"
                );
                
                // Persist task as running
                if let Err(e) = self.persist_task_status(&priority_task.task, TaskStatus::Running).await {
                    warn!(worker = %self.name, task_id = %task_id, error = %e, "Failed to persist task status");
                }
                
                match priority_task.task.execute(self.db.clone(), self.client.clone()).await {
                    Ok(_) => {
                        tasks_processed += 1;
                        info!(
                            worker = %self.name,
                            task_id = %task_id,
                            priority = ?priority,
                            tasks_processed = tasks_processed,
                            "Task completed"
                        );
                        
                        // Persist as completed
                        if let Err(e) = self.persist_task_status(&priority_task.task, TaskStatus::Completed).await {
                            warn!(worker = %self.name, task_id = %task_id, error = %e, "Failed to persist completion");
                        }
                    }
                    Err(e) => {
                        error!(
                            worker = %self.name,
                            task_id = %task_id,
                            priority = ?priority,
                            error = %e,
                            "Task failed"
                        );
                        
                        // Persist as failed
                        let status = TaskStatus::Failed { error: e.to_string() };
                        if let Err(e) = self.persist_task_status(&priority_task.task, status).await {
                            warn!(worker = %self.name, task_id = %task_id, error = %e, "Failed to persist failure");
                        }
                    }
                }
            }
        }
        
        info!(
            worker = %self.name,
            tasks_processed = tasks_processed,
            "Task queue worker stopped"
        );
        
        Ok(())
    }

    async fn persist_task_status(&self, task: &Box<dyn Task>, status: TaskStatus) -> Result<(), AppError> {
        let mut task_data = task.to_data();
        task_data.status = status;
        
        let collection = self.db.db().collection::<TaskData>("task_queue");
        
        let filter = mongodb::bson::doc! { "id": &task_data.id };
        let options = mongodb::options::ReplaceOptions::builder().upsert(true).build();
        
        collection.replace_one(filter, task_data).with_options(options)
            .await
            .map_err(|e| AppError::Module(format!("Failed to persist task: {}", e)))?;
        
        Ok(())
    }

    async fn load_persisted_tasks(&self, queue: &mut BinaryHeap<PriorityTask>) -> Result<(), AppError> {
        use mongodb::bson::doc;
        
        let collection = self.db.db().collection::<TaskData>("task_queue");
        
        // Find pending tasks
        let filter = doc! { "status": "Pending" };
        let mut cursor = collection.find(filter).await
            .map_err(|e| AppError::Module(format!("Failed to load tasks: {}", e)))?;
        
        let mut loaded_count = 0;
        
        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(task_data) => {
                    // Recreate task from data (you'll need to implement task factory)
                    info!(
                        worker = %self.name,
                        task_id = %task_data.id,
                        task_name = %task_data.name,
                        "Loaded persisted task"
                    );
                    loaded_count += 1;
                    // Note: You'll need to implement a task factory to recreate tasks from TaskData
                }
                Err(e) => {
                    warn!(worker = %self.name, error = %e, "Failed to deserialize task");
                }
            }
        }
        
        if loaded_count > 0 {
            info!(worker = %self.name, count = loaded_count, "Loaded persisted tasks");
        }
        
        Ok(())
    }
}