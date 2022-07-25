use std::{mem::{size_of, size_of_val}, ffi::c_void, ptr::{self, eq}};

use sysinfo::{System, SystemExt, ProcessExt, Pid, PidExt};
use windows::{Win32::{System::{Diagnostics::{ToolHelp::{CreateToolhelp32Snapshot, TH32CS_SNAPMODULE32, TH32CS_SNAPMODULE, MODULEENTRY32, Module32First, PROCESSENTRY32}, Debug::{ReadProcessMemory, WriteProcessMemory}}, Threading::{OpenProcess, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PROCESS_VM_WRITE, GetCurrentProcess, PROCESS_VM_OPERATION}, Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, VirtualProtect, PAGE_PROTECTION_FLAGS, VirtualProtectEx, VirtualAlloc, VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE}, ProcessStatus::{K32EnumProcessModulesEx, LIST_MODULES_32BIT, K32GetModuleBaseNameA, LIST_MODULES_ALL}, LibraryLoader::{LoadLibraryA, GetProcAddress}}, Foundation::{HANDLE, CloseHandle, GetLastError, WIN32_ERROR, HINSTANCE, FARPROC}, Graphics::Gdi::HDC}, core::PSTR};

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
                match OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION | PROCESS_VM_WRITE | PROCESS_VM_OPERATION, false, pid) {
                    Ok(handle) => return Ok(handle),
                    Err(_) => return Err("Unable to get handle to game process.".to_string())
                }
            }
            _ => Err("Unable to find game process when attempted to get handle".to_string())
        }
    }
}

pub fn get_module_handles(handle: HANDLE) -> Result<[HINSTANCE; 100], String> {
    let mut module_array: [HINSTANCE; 100] = [HINSTANCE::default(); 100];
    let module_array_ptr =  &mut module_array[0];
    let mut bytes_needed: u32 = 0;
    let size: u32 = (100*size_of::<HINSTANCE>()).try_into().unwrap();
    unsafe {
        if K32EnumProcessModulesEx(handle, &mut *module_array_ptr, size, &mut bytes_needed, LIST_MODULES_ALL).as_bool() {
            println!("Got module list, bytes needed: {}, bytes given: {}", bytes_needed, size);
            return Ok(module_array);
        }
    }
    Err(format!("Couldn't get module handles {:?}", bytes_needed))
}

pub fn get_module_base_name(handle: HANDLE, instance: HINSTANCE) -> Result<String, String> {
    let mut mod_name: [u8; 50] = [0; 50];
    unsafe {
        if K32GetModuleBaseNameA(handle, instance, &mut mod_name) != 0 {
            match std::str::from_utf8(&mod_name) {
                Ok(str) => return Ok(str.trim_matches(char::from(0)).to_string()),
                Err(_) => Err("Couldn't convert name to utf8".to_string()),
            }
        }
        else {
            Err("Couldn't get module name".to_string())
        }
    }
}

pub fn get_loaded_module(handle: HANDLE, name: String) -> Result<HINSTANCE, String> {
    match get_module_handles(handle) {
        Ok(instances) => {
            for module in instances.iter().filter(|x| x.0 != 0) {
                if get_module_base_name(handle, *module)? == name {
                    return Ok(*module)
                }
            }
            return Err(format!("Couldn't find loaded module named {}", name))
        }
        Err(e) => return Err(e)
    }
}

// Returns relative address of function from module

// pub fn get_exported_function_pointer(handle: HANDLE, module_name: &str, func_name: &str) -> Option<fn()> {
//     unsafe {
//         match LoadLibraryA(module_name) { // Load it in
//             Ok(instance) => {
//                 println!("Calling get_loaded with handle: {:?} and name: {:?} instance is {:X}", handle, module_name.to_string(), instance.0);
//                 let loaded_addr = get_loaded_module(handle, module_name.to_string())?;
//                 println!("Base module loaded at {:X}", instance.0);
//                 let func_relative_addr: usize = std::mem::transmute(GetProcAddress(instance, func_name));
//                 println!("fun_rec {:X}", func_relative_addr);
//                 let rel = func_relative_addr - loaded_addr.0 as usize;
//                 return Ok(rel)
//                 //return func_relative_addr - get_loaded_module(GetCurrentProcess(), module_name.to_string())
//             },
//             Err(e) => Err(e.to_string())
//         }
// 	}
// }

pub fn get_exported_function_address(handle: HANDLE, module_name: &str, func_name: &str) -> Option<usize> {
    unsafe {
        match LoadLibraryA(module_name) { 
            Ok(instance) => {
                println!("Calling get_loaded with handle: {:?} and name: {:?} instance is {:X}", handle, module_name.to_string(), instance.0);
                let pointer_address: usize = std::mem::transmute(GetProcAddress(instance, func_name));
                return Some(pointer_address)
            },
            _ => None
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

// TODO: Unify with write_mem_addr
pub fn write_float(handle: HANDLE, addr: usize, data: f32, buffer_size: i32) -> Result<bool, WIN32_ERROR> {
    let bytes: *mut usize = ptr::null_mut();
    let lp_buffer: *const c_void = <*const _>::cast(&data);
    unsafe {
        if WriteProcessMemory(handle, addr as *const c_void, lp_buffer, buffer_size as usize, bytes).as_bool() {
            return Ok(true);
        }
        else {
            Err(GetLastError())
        }
    }
}

// TODO: Unify with write_float
pub fn write_mem_addr(handle: HANDLE, addr: usize, data: usize, buffer_size: i32) -> Result<bool, WIN32_ERROR> {
    let bytes: *mut usize = ptr::null_mut();
    let lp_buffer: *const c_void = <*const _>::cast(&data);
    unsafe {
        if WriteProcessMemory(handle, addr as *const c_void, lp_buffer, buffer_size as usize, bytes).as_bool() {
            println!("[log] Wrote {:?} bytes to {:X}", bytes, addr);
            return Ok(true);
        }
        else {
            Err(GetLastError())
        }
    }
}

//TODO: enum semantics

/// Resolves a chain of pointers to the final address or the contents of the final address. Necessary for finding the location of 
/// values in memory when games use dynamic memory allocation
///
/// # Arguments
///
/// * `handle` - A handle to the game process, which must have read access.
/// * `base_addr` - The base address of the game process or module.
/// * `offsets` - An array containing the hexadecimal offsets required for each level in the chain.
/// * `addr_type` - The type of address at the end of the chain. For some data structures, it may be desirable to pass the Value type and stop 1 level below.
///
/// # Returns
/// 
/// If the pointer chain could be resolved, `Some()` will contain the memory address at the end of the chain.
/// If unsuccessful, `None` will be returned

pub fn resolve_pointer_chain(handle: HANDLE, base_addr: usize, offsets: &[usize], addr_type: AddressType) -> Option<usize> {
    let mut final_addr = base_addr;
    match addr_type {
        AddressType::Value => {
            for o in offsets {
                if let Some(resolved) = read_mem_addr(handle, final_addr + o, 4) {
                    final_addr = resolved;
                }
                else { return None }
            }
            return Some(final_addr)
        },
        AddressType::Pointer => {
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

/// Patches a NOP instruction at the memory address provided
///
/// # Arguments
///
/// * `handle` - A handle to the game process, which must have write access
/// * `addr` - A memory address at which to patch the NOP instruction
///
/// # Returns
/// 
/// If the patch was successful, `Ok()` will contain a message indicating the address that was patched.
/// If unsuccessful, `Err()` will return the windows error code encountered when trying to call `WriteProcessMemory`

pub fn nop_address(handle: HANDLE, addr: usize) -> Result<String, WIN32_ERROR> {
    match write_mem_addr(handle, addr, 0x90, 1) {
        Ok(_) => return Ok(format!("NOP instruction placed at {:X}", addr)),
        Err(e) => Err(e)
    }
}

pub fn detour(handle: HANDLE, jmp_from: usize, jmp_to: usize) -> Result<usize, WIN32_ERROR> {
    let relative_jmp = jmp_to - jmp_from - 5;
    write_mem_addr(handle, jmp_from, 0xE9, 1)?;
    write_mem_addr(handle, jmp_from + 1, relative_jmp, 4)?;
    return Ok(jmp_to)
}


pub fn trampoline_hook(handle: HANDLE, target_func: usize, src_func: usize, bytes: i32) -> Result<usize, WIN32_ERROR> {
    unsafe {
        let gateway = VirtualAllocEx(handle, ptr::null(), bytes as usize, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as usize;
        write_mem_addr(handle, gateway, target_func, bytes)?;
        let gateway_rel = (target_func - gateway - 5) as usize;
        write_mem_addr(handle, gateway + bytes as usize, 0xE9, 1)?; 
        write_mem_addr(handle, gateway + bytes as usize + 0x1, gateway_rel, 4)?;
        return detour(handle, target_func, src_func);
    }
}

// VirtualProtectEx(handle, addr as *const c_void, 487424, PAGE_PROTECTION_FLAGS(4), &mut old_proc_flags);
// VirtualProtectEx(handle, addr as *const c_void, 487424, old_proc_flags, &mut old_proc_flags);
// let mut mbi = MEMORY_BASIC_INFORMATION::default();
// let buff = size_of_val(&mbi);
// unsafe {
//     VirtualQueryEx(handle, addr as *const c_void, &mut mbi, buff);
// }
// println!("MBI {:?}", mbi);

