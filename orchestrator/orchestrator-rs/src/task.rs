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

#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
pub enum State {
    #[default]
    Pending,
    Scheduled,
    Completed,
    Running,
    Failed,
}

#[allow(dead_code)]
pub fn contains(src: &State, dst: &State) -> bool {
    let state_transition_map: HashMap<State, Vec<State>> = HashMap::from([
        (State::Pending, vec![State::Scheduled]),
        (
            State::Scheduled,
            vec![State::Scheduled, State::Failed, State::Running],
        ),
        (State::Completed, vec![]),
        (
            State::Running,
            vec![State::Running, State::Completed, State::Failed],
        ),
        (State::Failed, vec![]),
    ]);
    if !state_transition_map.contains_key(src) {
        return false;
    }
    if let Some(st) = state_transition_map.get(src) {
        st.contains(dst)
    } else {
        false
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Task<T>
where
    T: Into<String> + Eq + Hash,
{
    pub id: Uuid,
    pub container_id: Option<T>,
    pub name: T,
    pub state: State,
    pub image: T,
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
    pub name: T,
    pub attach_stdin: Option<bool>,
    pub attach_stdout: Option<bool>,
    pub attach_stderr: Option<bool>,
    pub exposed_ports: Option<HashMap<T, HashMap<(), ()>>>,
    pub cmd: Option<Vec<T>>,
    pub image: T,
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
    pub fn new(name: &str, image: &str, env: Option<Vec<String>>) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            env,
            ..Default::default()
        }
    }
}

impl DockerClient<String> {
    pub fn new(config: Config<String>) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self {
            client: docker,
            config,
            container_id: None,
        })
    }

    pub async fn run(&self) -> Result<DockerResult<String>, Box<dyn std::error::Error + 'static>> {
        let mut image = self.config.image.as_str();
        if image.is_empty() {
            image = "alpine:3"
        }

        self.client
            .create_image::<&str>(
                Some(CreateImageOptions {
                    from_image: image,
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
