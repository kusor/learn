#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub ip: String,
    pub memory: u64,
    pub memory_allocated: u64,
    pub disk: u64,
    pub cores: u32,
    pub disk_allocated: u64,
    pub task_count: u32,
    pub role: String,
}
