use bollard::errors::Error;
use chrono::prelude::*;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[path = "../node.rs"]
mod node;
#[path = "../task.rs"]
mod task;
#[path = "../worker.rs"]
mod worker;

#[allow(dead_code)]
async fn create_container(
) -> Result<task::DockerClient<String>, Box<dyn std::error::Error + 'static>> {
    let c = task::Config::new("test-container-1", "alpine:3", None);
    let mut dc = task::DockerClient::new(c)?;
    let dr = dc.run().await?;
    dc.container_id = dr.container_id;
    Ok(dc)
}

async fn stop_container(
    dc: &task::DockerClient<String>,
) -> Result<task::DockerResult<String>, Box<dyn std::error::Error + 'static>> {
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

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Hello orchestrator!");

    let t = task::Task {
        id: Uuid::new_v4(),
        name: "Task-1".to_string(),
        state: task::State::Pending,
        image: "Image-1".to_string(),
        memory: Some(1024),
        disk: Some(1),
        restart_policy: Some("always".to_string()),
        ..Default::default()
    };

    log::info!("task: {:#?}\n", &t);

    let te = task::TaskEvent {
        id: Uuid::new_v4(),
        state: task::State::Pending,
        timestamp: Utc::now(),
        task: t,
    };

    log::info!("task event: {:#?}\n", te);

    let n = node::Node {
        name: "Node-1".to_string(),
        ip: "192.168.1.1".to_string(),
        cores: 4,
        memory: 1024,
        disk: 25,
        task_count: 0,
        disk_allocated: 0,
        memory_allocated: 0,
        role: "worker".to_string(),
    };

    log::info!("node: {:#?}\n", n);
    let mut w = worker::Worker::new(Uuid::new_v4().to_string());
    let tw = task::Task {
        id: Uuid::new_v4(),
        name: "test-container-1-rs".to_string(),
        image: "strm/helloworld-http".to_string(),
        ..Default::default()
    };
    w.add_task(tw);

    // TODO: Modify to use w.run_task method and remove these top level fns
    match create_container().await {
        Err(error) => {
            log::error!("Failed to create the container: {:#?}\n", error);
        }
        Ok(dc) => {
            sleep(Duration::from_secs(5)).await;
            log::info!("created container with id: {:#?}\n", &dc.container_id);

            match stop_container(&dc).await {
                Err(error) => {
                    log::error!("Failed to stop the container: {:#?}\n", error);
                }
                Ok(dr) => {
                    if dr.error.is_some() {
                        log::error!("Failed to stop the container: {:#?}\n", dr.error);
                    } else {
                        log::info!("Container successfully removed");
                    }
                }
            };
        }
    };
}
