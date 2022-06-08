use sysinfo::{System, SystemExt, ProcessExt};

pub fn get_process_list() -> Vec<String> {
    let mut system_info = System::new_all();
    system_info.refresh_all();
    let mut processes = vec![];
    for (_pid, process) in system_info.processes() {
        processes.push(process.name().to_string());
    }
    return processes;
}
