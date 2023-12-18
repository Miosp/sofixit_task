#[macro_export]
macro_rules! measure {
    ($func:expr) => {
        {
            use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
            use perf_monitor::{cpu::ProcessStat, mem::get_process_memory_info};
    
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
                let mut cpu = ProcessStat::cur().unwrap();
                let memory_usage = get_process_memory_info().unwrap().resident_set_size;
                
                std::thread::sleep(std::time::Duration::from_millis(200));

                cpu_util.push(cpu.cpu().unwrap() as f32);
                memory_util.push(memory_usage);
            }
    
            (handle.join().unwrap(), cpu_util, memory_util)
        }
    };
}

#[macro_export]
macro_rules! measure_async {
    ($func:expr) => {
        {
            use std::sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}};
            use perf_monitor::{cpu::ProcessStat, mem::get_process_memory_info};
    
            let running = Arc::new(AtomicBool::new(true));
            let r = Arc::clone(&running);
            let mut cpu_util = vec![];
            let mut memory_util = vec![];
            
            let handle = tokio::spawn(async move {
                let result = $func.await;
                r.store(false, SeqCst);
                result
            });
            
            while running.load(SeqCst) {
                let mut cpu = ProcessStat::cur().unwrap();
                let memory_usage = get_process_memory_info().unwrap().resident_set_size;
                
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                cpu_util.push(cpu.cpu().unwrap() as f32);
                memory_util.push(memory_usage);
            }
    
            (handle.await.unwrap(), cpu_util, memory_util)
        }
    };
}