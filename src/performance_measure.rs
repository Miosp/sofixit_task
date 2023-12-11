#[macro_export]
macro_rules! measure {
    ($func:expr) => {
        {
            use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
            use sysinfo::{SystemExt, System, Pid, ProcessExt, ProcessRefreshKind};
            use std::process::id;
    
            let mut sys = System::new_all();
            sys.refresh_process_specifics(Pid::from(id() as usize), ProcessRefreshKind::everything().without_disk_usage().without_user());
            let running = Arc::new(AtomicBool::new(true));
            let r = Arc::clone(&running);
            let mut cpu_util = vec![];
            let mut memory_util = vec![];
            
            let handle = std::thread::spawn(move || {
                let result = $func;
                r.store(false, SeqCst);
                result
            });
            
            while running.load(SeqCst) {
                sys.refresh_process_specifics(Pid::from(id() as usize), ProcessRefreshKind::everything().without_disk_usage().without_user());
                let process = sys.process(Pid::from(id() as usize)).unwrap();
                cpu_util.push(process.cpu_usage());
                memory_util.push(process.memory());
                
                // We are not able to get more precise data than 200ms
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
    
            (handle.join().unwrap(), cpu_util, memory_util)
        }
    };
}