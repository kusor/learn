use chrono::prelude::*;
use uuid::Uuid;

pub type StateVal = u32;

#[derive(Debug, Clone)]
pub enum State {
    Pending(StateVal),
    //     Scheduled(StateVal),
    //     Completed(StateVal),
    //     Running(StateVal),
    //     Failed(StateVal),
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub state: State,
    pub image: String,
    pub memory: u64,
    pub disk: u64,
    // ExposedPorts  nat.PortSet
    // PortBindings  map[string]string
    pub restart_policy: String,
}

#[derive(Debug, Clone)]
pub struct TaskEvent<'a> {
    pub id: Uuid,
    pub state: State,
    pub timestamp: DateTime<Utc>,
    pub task: &'a Task,
}
