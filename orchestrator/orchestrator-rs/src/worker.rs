use crate::task::{self, Task};
use bollard::errors::Error;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Worker {
    pub name: Option<String>,
    pub queue: VecDeque<Task<String>>,
    pub db: Arc<RwLock<HashMap<uuid::Uuid, Task<String>>>>,
    pub task_count: usize,
}

impl Worker {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
            queue: VecDeque::new(),
            db: Arc::new(RwLock::new(HashMap::new())),
            task_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn collect_stats(&self) {}

    #[allow(dead_code)]
    pub fn add_task(&mut self, t: Task<String>) {
        self.queue.push_back(t)
    }

    #[allow(dead_code)]
    pub async fn run_task(
        &mut self,
    ) -> Result<task::DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        match self.queue.pop_front() {
            None => Ok(task::DockerResult {
                action: "run".to_string(),
                container_id: None,
                error: None,
                result: None,
            }),
            Some(t) => {
                let reader = self.db.read().await;
                let mut persisted = reader.get(&t.id);
                let t_persisted = persisted.get_or_insert(&t);
                if task::contains(&t_persisted.state, &t.state) {
                    match t.state {
                        task::State::Scheduled => return self.start_task(t).await,
                        task::State::Completed => return self.stop_task(t).await,
                        _ => {
                            return Ok(task::DockerResult {
                                action: "run".to_string(),
                                container_id: None,
                                error: Some(Error::DockerResponseServerError {
                                    status_code: 422,
                                    message: "Invalid task State".to_string(),
                                }),
                                result: None,
                            })
                        }
                    }
                }
                Ok(task::DockerResult {
                    action: "run".to_string(),
                    container_id: None,
                    error: None,
                    result: None,
                })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn start_task(
        &self,
        mut t: Task<String>,
    ) -> Result<task::DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        let config = task::Config::new(&t.name, &t.image, None);
        let dc = task::DockerClient::new(config)?;
        let dr = dc.run().await?;
        if dr.error.is_some() {
            log::info!("Error running task: {:#?}: {:#?}", &t.id, &dr.container_id);
            t.state = task::State::Failed;
            self.db.write().await.insert(t.id.clone(), t);
        } else {
            t.state = task::State::Running;
            t.container_id = dr.container_id.clone();
            self.db.write().await.insert(t.id.clone(), t);
        }
        Ok(dr)
    }

    #[allow(dead_code)]
    pub async fn stop_task(
        &self,
        t: Task<String>,
    ) -> Result<task::DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        let config = task::Config::new(&t.name, &t.image, None);
        let dc = task::DockerClient::new(config)?;

        if let Some(container_id) = &dc.container_id {
            let dr = dc.stop(container_id).await?;
            Ok(dr)
        } else {
            Ok(task::DockerResult {
                error: Some(Error::DockerResponseServerError {
                    status_code: 422,
                    message: "Missing container id".to_string(),
                }),
                action: "stop".to_string(),
                container_id: None,
                result: Some("failed".to_string()),
            })
        }
    }
}
