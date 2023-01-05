use std::collections::HashMap;
use std::hash::Hash;

use bollard::{
    container::Config as ContainerConfig, container::RemoveContainerOptions as RemoveOptions,
    container::StopContainerOptions as StopOptions, errors::Error, image::CreateImageOptions,
    Docker,
};
use futures_util::TryStreamExt;

use chrono::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum State {
    #[default]
    Pending,
    //     Scheduled,
    //     Completed,
    //     Running,
    //     Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Task<T>
where
    T: Into<String> + Eq + Hash,
{
    pub id: Uuid,
    pub name: Option<T>,
    pub state: State,
    pub image: Option<T>,
    pub memory: Option<u64>,
    pub disk: Option<u64>,
    // Not absolutely sure of the format of these, we'll see
    pub exposed_ports: Option<HashMap<T, HashMap<(), ()>>>,
    pub port_bindings: Option<HashMap<T, T>>,
    pub restart_policy: Option<T>,
    pub start_time: Option<DateTime<Utc>>,
    pub finish_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskEvent<T>
where
    T: Into<String> + Eq + Hash,
{
    pub id: Uuid,
    pub state: State,
    pub timestamp: DateTime<Utc>,
    pub task: Task<T>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Config<T>
where
    T: Into<String> + Eq + Hash,
{
    pub name: Option<T>,
    pub attach_stdin: Option<bool>,
    pub attach_stdout: Option<bool>,
    pub attach_stderr: Option<bool>,
    pub exposed_ports: Option<HashMap<T, HashMap<(), ()>>>,
    pub cmd: Option<Vec<T>>,
    pub image: Option<T>,
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
    pub disk: Option<u64>,
    pub env: Option<Vec<T>>,
    pub restart_policy: Option<T>,
}

#[derive(Debug, Clone)]
pub struct DockerClient<T>
where
    T: Into<String> + Eq + Hash,
{
    pub client: Docker,
    pub config: Config<T>,
    pub container_id: Option<T>,
}
#[allow(dead_code)]
pub struct DockerResult<T> {
    pub error: Option<Error>,
    pub action: T,
    pub container_id: Option<T>,
    pub result: Option<T>,
}

impl DockerResult<String> {
    pub fn new(
        error: Option<Error>,
        action: String,
        container_id: Option<String>,
        result: Option<String>,
    ) -> Self {
        Self {
            error,
            action,
            container_id,
            result,
        }
    }
}

impl Config<String> {
    pub fn new(name: String, image: String, env: Option<Vec<String>>) -> Self {
        Self {
            name: Some(name),
            image: Some(image),
            env,
            ..Default::default()
        }
    }
}

impl DockerClient<String> {
    #[allow(dead_code)]
    pub fn new(config: Config<String>) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self {
            client: docker,
            config,
            container_id: None,
        })
    }

    #[allow(dead_code)]
    pub async fn run(&self) -> Result<DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        let image: String = match &self.config.image {
            Some(img) => img.to_string(),
            None => "alpine:3".to_string(),
        };

        self.client
            .create_image::<&str>(
                Some(CreateImageOptions {
                    from_image: image.as_str(),
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;

        let container_id = self
            .client
            .create_container::<&str, &str>(
                None,
                ContainerConfig {
                    image: Some(&image),
                    tty: Some(true),
                    ..Default::default()
                },
            )
            .await?
            .id;

        self.client
            .start_container::<String>(&container_id, None)
            .await?;

        Ok(DockerResult::new(
            None,
            "start".to_string(),
            Some(container_id),
            Some("success".to_string()),
        ))
    }

    #[allow(dead_code)]
    pub async fn stop(
        &self,
        container_id: &str,
    ) -> Result<DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        self.client
            .stop_container(container_id, Some(StopOptions { t: 15 }))
            .await?;
        self.client
            .remove_container(
                container_id,
                Some(RemoveOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;
        Ok(DockerResult::new(
            None,
            "stop".to_string(),
            Some(container_id.to_string()),
            Some("success".to_string()),
        ))
    }
}
