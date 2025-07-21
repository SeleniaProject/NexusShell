use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Completed(i32),
}

#[derive(Debug)]
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
        self.map.get(&id).map(|r| r.clone())
    }

    pub fn list(&self) -> Vec<Job> {
        self.map.iter().map(|r| r.clone()).collect()
    }
}

pub static JOB_TABLE: Lazy<JobTable> = Lazy::new(|| {
    let table = JobTable::default();
    // Start reaper thread only on Unix for now
    #[cfg(unix)]
    start_sigchld_reaper();
    table
});

#[cfg(unix)]
fn start_sigchld_reaper() {
    use signal_hook::consts::signal::SIGCHLD;
    use signal_hook::iterator::Signals;
    use std::thread;
    use std::time::Duration;
    use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

    thread::spawn(|| {
        let mut signals = Signals::new(&[SIGCHLD]).expect("create signal iterator");
        for _ in &mut signals {
            // Drain child statuses
            loop {
                match waitpid(-1, Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::Exited(pid, code)) => {
                        JOB_TABLE.update_state(pid as u32, JobState::Completed(code));
                    }
                    Ok(WaitStatus::StillAlive) => break,
                    Ok(_) => break,
                    Err(_) => break,
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}

#[cfg(windows)]
fn start_sigchld_reaper() {}

pub fn init() {
    Lazy::force(&JOB_TABLE);
} 