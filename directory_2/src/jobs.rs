use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use lazy_static::lazy_static;

pub struct Job {
    pub id: u32,
    pub command: String,
    pub child: Arc<TokioMutex<tokio::process::Child>>,
}

lazy_static! {
    pub static ref JOB_REGISTRY: Mutex<Vec<Job>> = Mutex::new(Vec::new());
}

static NEXT_JOB_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

pub fn add_job(command: String, child: Arc<TokioMutex<tokio::process::Child>>) -> u32 {
    let id = NEXT_JOB_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    JOB_REGISTRY.lock().unwrap().push(Job {
        id,
        command,
        child,
    });
    id
}

pub fn remove_job(id: u32) {
    if let Ok(mut registry) = JOB_REGISTRY.lock() {
        registry.retain(|j| j.id != id);
    }
}

pub fn get_job(id: u32) -> Option<Job> {
    if let Ok(registry) = JOB_REGISTRY.lock() {
        registry.iter().find(|j| j.id == id).map(|j| Job {
            id: j.id,
            command: j.command.clone(),
            child: j.child.clone(),
        })
    } else {
        None
    }
}

pub fn list_jobs() -> Vec<(u32, String)> {
    if let Ok(registry) = JOB_REGISTRY.lock() {
        registry.iter().map(|j| (j.id, j.command.clone())).collect()
    } else {
        Vec::new()
    }
}
