use std::{mem::size_of, ffi::c_void, ptr};

use sysinfo::{System, SystemExt, ProcessExt, Pid, PidExt};
use windows::Win32::{System::{Diagnostics::{ToolHelp::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE32, TH32CS_SNAPMODULE, MODULEENTRY32, Module32First, PROCESSENTRY32}, Debug::ReadProcessMemory}, Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION}}, Foundation::{HANDLE, CloseHandle}};

pub enum AddressType {
    Pointer,
    Value,
}

pub fn get_process_list() -> Vec<(Pid, String)> {
    let mut system_info = System::new_all();
    system_info.refresh_all();
    let mut processes = vec![];
    for (_, process) in system_info.processes() {
        processes.push((process.pid(), process.name().to_string()));
    }    
    return processes;
}

pub fn get_process_handle(pid: u32) -> Result<HANDLE, String> {
    let mut entry = PROCESSENTRY32::default();
    entry.dwSize = size_of::<PROCESSENTRY32>() as u32;
    unsafe {
        match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) {
            Ok(_handle) => {
                match OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false, pid) {
                    Ok(handle) => return Ok(handle),
                    Err(_) => return Err("Unable to get handle to game process.".to_string())
                }
            }
            _ => Err("Unable to find game process when attempted to get handle".to_string())
        }
    }
}

pub fn get_process_pid(name: &str) -> Result<u32, String> {
    for (p_id, p_name) in get_process_list() {
        if p_name == name {
            return Ok(p_id.as_u32())
        }
    }
    return Err("Couldn't find target process".to_string());
}


pub fn get_process_address(pid: u32) -> Result<usize, String> {
    let mut mod_entry = MODULEENTRY32::default();
    mod_entry.dwSize = size_of::<MODULEENTRY32>() as u32;
    unsafe {
        match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) {
            Ok(snapshot) => {
                if Module32First(snapshot, &mut mod_entry).as_bool() {
                    CloseHandle(snapshot);
                    return Ok(mod_entry.modBaseAddr as usize);
                }
                else { return Err("Couldn't find target process".to_string()) }
            }
            _ => { return Err("Couldn't find target process".to_string()) }
        }
    }
}

pub fn read_mem_addr(handle: HANDLE, addr: usize, buffer_size: i8) -> Option<usize> {
    let mut data: *mut c_void = ptr::null_mut();
    let lp_buffer: *mut c_void = <*mut _>::cast(&mut data);
    //let nsize = size_of::<usize>();
    let mut bytes: *mut usize = ptr::null_mut();
    let lp_bytes_read: *mut usize = <*mut _>::cast(&mut bytes);
    unsafe {
        if ReadProcessMemory(handle, addr as *const c_void, lp_buffer, buffer_size as usize, lp_bytes_read).as_bool() { return Some(data as usize) }
        else { None }
    }
}

pub fn resolve_pointer_chain(handle: HANDLE, base_addr: usize, offsets: &[usize], addr_type: AddressType) -> Option<usize> {
    let mut final_addr = base_addr;
    match addr_type {
        AddressType::Pointer => {
            for o in offsets {
                if let Some(resolved) = read_mem_addr(handle, final_addr + o, 4) {
                    final_addr = resolved;
                }
                else { return None }
            }
            return Some(final_addr)
        },
        AddressType::Value => {
            for o in &offsets[..offsets.len() - 1] {
                if let Some(resolved) = read_mem_addr(handle, final_addr + o, 4) {
                    final_addr = resolved;
                }
                else { return None }
            }
            Some(final_addr + &offsets[offsets.len() - 1])

        },
    }
}


