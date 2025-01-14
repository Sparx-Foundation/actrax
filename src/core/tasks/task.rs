use rand::random;
use serde::{Deserialize, Serialize};


/// Represents the state of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    WorkingOn,
    Completed,
    Failed,
}

/// Represents different types of operations that can be performed by a client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    /// DLL injection to a specific process
    Inject { dll_name: String, process_id: u32 },
    /// Run a PowerShell script (.ps1)
    RunPowerShell { script_path: String },
    /// Execute a batch file (.bat)
    ExecuteBatch { batch_file_path: String },
}

/// A struct to represent a single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub status: TaskStatus,
    pub client_id: i32,
    pub operation: OperationType,
}

impl Task {
    /// Creates a new task with a random numeric ID and a specified operation.
    pub fn new(client_id: i32, description: String, operation: OperationType) -> Self {
        let id = random::<u64>();
        Self {
            id,
            description,
            status: TaskStatus::Pending,
            client_id,
            operation,
        }
    }
}
