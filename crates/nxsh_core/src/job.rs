use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Completed(i32),
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u32,
    pub pid: u32,
    pub cmd: String,
    pub state: JobState,
}

#[derive(Debug, Default)]
pub struct JobTable {
    map: DashMap<u32, Job>,
    next_id: Mutex<u32>,
}

impl JobTable {
    pub fn add_job(&self, pid: u32, cmd: String) -> u32 {
        let mut guard = self.next_id.lock().unwrap();
        let id = *guard;
        *guard += 1;
        let job = Job { id, pid, cmd, state: JobState::Running };
        self.map.insert(id, job);
        id
    }

    pub fn update_state(&self, pid: u32, new_state: JobState) {
        if let Some(mut entry) = self.map.iter_mut().find(|e| e.pid == pid) {
            entry.state = new_state;
        }
    }

    pub fn get(&self, id: u32) -> Option<Job> {
        self.map.get(&id).map(|r| r.value().clone())
    }

    pub fn list(&self) -> Vec<Job> {
        self.map.iter().map(|r| r.value().clone()).collect()
    }

    pub fn disown_all(&self) {
        self.map.clear();
    }
}

static JOB_TABLE: Lazy<JobTable> = Lazy::new(JobTable::default);

pub fn init() {
    Lazy::force(&JOB_TABLE);
    
    #[cfg(unix)]
    {
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
        use nix::unistd::Pid;
        use std::thread;
        
        thread::spawn(|| loop {
            match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    let pid_as_u32 = pid.as_raw() as u32;
                    JOB_TABLE.update_state(pid_as_u32, JobState::Completed(code));
                }
                Ok(WaitStatus::Stopped(pid, _sig)) => {
                    let pid_as_u32 = pid.as_raw() as u32;
                    JOB_TABLE.update_state(pid_as_u32, JobState::Stopped);
                }
                Ok(WaitStatus::Signaled(pid, _sig, _)) => {
                    let pid_as_u32 = pid.as_raw() as u32;
                    JOB_TABLE.update_state(pid_as_u32, JobState::Completed(-1));
                }
                Ok(_) => {
                    // Other status types or no children
                }
                Err(_) => {
                    // No children to wait for or other error
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        });
    }
}

pub fn add_job(pid: u32, cmd: String) -> u32 {
    JOB_TABLE.add_job(pid, cmd)
}

pub fn get_job(id: u32) -> Option<Job> {
    JOB_TABLE.get(id)
}

pub fn list_jobs() -> Vec<Job> {
    JOB_TABLE.list()
}

pub fn disown_all() {
    JOB_TABLE.disown_all();
} 