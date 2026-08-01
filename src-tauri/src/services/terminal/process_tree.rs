use std::sync::atomic::{AtomicBool, Ordering};

use portable_pty::{Child, MasterPty};

use super::types::TerminalServiceError;

pub struct ProcessTreeGuard {
    terminated: AtomicBool,
    #[cfg(windows)]
    job: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(unix)]
    process_group: libc::pid_t,
}

unsafe impl Send for ProcessTreeGuard {}
unsafe impl Sync for ProcessTreeGuard {}

impl ProcessTreeGuard {
    pub fn attach(child: &dyn Child, master: &dyn MasterPty) -> Result<Self, TerminalServiceError> {
        #[cfg(windows)]
        {
            let _ = master;
            Self::attach_windows(child)
        }
        #[cfg(unix)]
        {
            let _ = child;
            let process_group = master
                .process_group_leader()
                .ok_or(TerminalServiceError::SpawnFailed("process_group"))?;
            Ok(Self {
                terminated: AtomicBool::new(false),
                process_group,
            })
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = (child, master);
            Err(TerminalServiceError::SpawnFailed("platform"))
        }
    }

    #[cfg(windows)]
    fn attach_windows(child: &dyn Child) -> Result<Self, TerminalServiceError> {
        use std::mem::size_of;
        use std::ptr;

        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let process = child
            .as_raw_handle()
            .ok_or(TerminalServiceError::SpawnFailed("process_handle"))?
            as windows_sys::Win32::Foundation::HANDLE;
        let job = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if job.is_null() {
            return Err(TerminalServiceError::SpawnFailed("job_create"));
        }
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let assigned = configured != 0 && unsafe { AssignProcessToJobObject(job, process) } != 0;
        if !assigned {
            unsafe { CloseHandle(job) };
            return Err(TerminalServiceError::SpawnFailed("job_assign"));
        }
        Ok(Self {
            terminated: AtomicBool::new(false),
            job,
        })
    }

    pub fn terminate(&self) {
        if self.terminated.swap(true, Ordering::AcqRel) {
            return;
        }
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1);
        }
        #[cfg(unix)]
        {
            // Interactive shells can place background jobs in their own process
            // groups within the PTY session. Give the shell a brief chance to
            // forward SIGHUP to those jobs before force-killing its own group.
            unsafe {
                libc::kill(-self.process_group, libc::SIGHUP);
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
            unsafe {
                libc::kill(-self.process_group, libc::SIGKILL);
            }
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessTreeGuard {
    fn drop(&mut self) {
        if !self.job.is_null() {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };
            self.job = std::ptr::null_mut();
        }
    }
}

#[cfg(unix)]
impl Drop for ProcessTreeGuard {
    fn drop(&mut self) {
        self.terminate();
    }
}
