use chrono::prelude::*;
use uuid::Uuid;

#[path = "../node.rs"]
mod node;
#[path = "../task.rs"]
mod task;

fn main() {
    println!("Hello orchestrator!");

    let t = task::Task {
        id: Uuid::new_v4(),
        name: "Task-1".to_string(),
        state: task::State::Pending(0),
        image: "Image-1".to_string(),
        memory: 1024,
        disk: 1,
        restart_policy: "always".to_string(),
    };

    let te = task::TaskEvent {
        id: Uuid::new_v4(),
        state: task::State::Pending(0),
        timestamp: Utc::now(),
        task: &t,
    };

    println!("task: {:#?}\n", t);
    println!("task event: {:#?}\n", te);

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

    println!("node: {:#?}\n", n)
}
