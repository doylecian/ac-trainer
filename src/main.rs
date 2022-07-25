#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case_types)]

mod memory;
mod structs;
mod math;

use core::time;
use std::{self, thread, intrinsics::transmute};
use memory::{get_process_pid, get_process_address, write_float, nop_address, get_module_handles, get_module_base_name, get_loaded_module, write_mem_addr, detour, trampoline_hook};
use structs::{Game, EntityList};
use windows::Win32::{Foundation::{CloseHandle, HANDLE, HINSTANCE, GetLastError, FARPROC}, UI::Input::KeyboardAndMouse::GetAsyncKeyState, Graphics::Gdi::HDC, System::{LibraryLoader::{GetProcAddress, LoadLibraryA}, Threading::{GetCurrentProcess, GetExitCodeProcess}}};

use crate::{memory::{get_process_handle, resolve_pointer_chain, read_mem_addr, AddressType, get_exported_function_address}, math::euclid_dist};

const PROCESS_NAME: &str = "ac_client.exe";
const PLAYER_BASE: usize = 0x00109B74;
const PLAYER_HEALTH_0FFSETS: [usize; 2] = [PLAYER_BASE, 0xF8];
const PLAYER_VIEW_YAW_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0040];
const PLAYER_VIEW_PITCH_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0044];
const ENTITY_LIST_START: usize = 0x10f4f8;
const ENTITY_COORDS_OFFSET: [usize; 3] = [0x4, 0x8, 0xc];
const ENTITY_NAME_OFFSET: usize = 0x224;
const PLAYER_COUNT_OFFSET: usize = 0x10F500;
const Y_VIEW_OFFSETS: [usize; 5] = [0x0010A280, 0x8, 0x214, 0x70, 0x6c];

//type wglSwapBuffers_t = Option<unsafe extern "system" fn(hdc: HDC) -> bool>;

fn main() {

	let (game_handle, proc_addr) = process_init("ac_client.exe").expect("Cannot initialise game hack - make sure the game is running");
	let mut client = Game::new(proc_addr, game_handle);

	if let Some(v) = resolve_pointer_chain(client.proc_handle, client.base_address, &Y_VIEW_OFFSETS, AddressType::Value) {
		client.add_address("viewTable".to_string(), v);
	}

	// if let Some(ent_list_addr) = resolve_pointer_chain(client.proc_handle, client.base_address, &[ENTITY_LIST_START], AddressType::Value) {
	// 	let mut ent_list = EntityList::new(ent_list_addr, 0x4, 5, client.proc_handle);
	// 	ent_list.populate(5);
	// 	client.add_entity_list("Bot player list", ent_list);
	// }

	//jmp_address(client.proc_handle, 0x4637F4, 0x463800).expect("Error");

	// List games imported modules
	match get_module_handles(client.proc_handle) {
		Ok(handles) => {
			for instance in handles.iter().filter(|x| x.0 != 0) {
				let name = get_module_base_name(client.proc_handle, *instance).unwrap();
				println!("{:X} - {:?}", instance.0, name);
			}
		}
		Err(msg) => panic!("{}", msg)
	}

	unsafe {
		let wglSwapBuffers_o = get_exported_function_address(client.proc_handle, "opengl32.dll", "wglSwapBuffers");
		match wglSwapBuffers_o {
			Some(addr) => {
				println!("Found wglSwapBuffers address -> {:X}", addr);
				let maddr = wglSwapBuffers_h as usize;
				println!("My function address -> {:X}", maddr);
			}
    		None =>println!("Error"),
		}
	}


	// unsafe {
	// 	let wgl_swap_buffers_offset = get_exported_function_offset(GetCurrentProcess(), "opengl32.dll", "wglSwapBuffers").unwrap();
	// 	println!("Found wglSwapBuffers offset -> {:X}", wgl_swap_buffers_offset);
	// }

	//detour(client.proc_handle, 0x4637f7, 0x463808); // 8 byte jump

	loop {
		update(&mut client);
		thread::sleep(time::Duration::from_millis(10));
		unsafe {
			if GetAsyncKeyState(0x02) != 0 {
				match write_float(client.proc_handle, client.value_pointers["viewTable"] + 0x248, 123.0, 4) {
					Ok(_) => {
						println!("Changed view yaw");
						match write_float(client.proc_handle, client.value_pointers["viewTable"] + 0x24C, 0.0, 4) {
							Ok(_) => println!("Changed view pitch"),
							Err(_) => println!("Couldn't write to view angle address!"),
						}
					},
					Err(msg) => println!("Error message: {:?}", msg)
				}
			}
			if GetAsyncKeyState(0x10) != 0 {
				let mut exit_code_buffer = [0u32; 1];
				GetExitCodeProcess(client.proc_handle, exit_code_buffer.as_mut_ptr());
				println!("{:?}, Exit code: {:?}", client.proc_handle, exit_code_buffer);
				CloseHandle(client.proc_handle);
				let mut exit_code_buffer = [0u32; 1];
				GetExitCodeProcess(client.proc_handle, exit_code_buffer.as_mut_ptr());
				println!("{:?}, Exit code: {:?}", client.proc_handle, exit_code_buffer);
				return;
			}
		}
	}
}

unsafe extern "system" fn wglSwapBuffers_h(hdc: HDC) -> bool  {
	return true
	//return wglSwapBuffers_o.unwrap()(hdc)
}


fn update(client: &mut Game) {		
	client.show_entity_lists();
	let local_player_xpos_addr = read_mem_addr(client.proc_handle, client.base_address + PLAYER_BASE, 4).unwrap();
	let local_player_xpos = read_mem_addr(client.proc_handle, local_player_xpos_addr + ENTITY_COORDS_OFFSET[0], 4).unwrap();
	let local_player_ypos = read_mem_addr(client.proc_handle, local_player_xpos_addr + ENTITY_COORDS_OFFSET[1], 4).unwrap();
	let x_view = read_mem_addr(client.proc_handle, client.value_pointers["viewTable"] + 0x248, 4).unwrap();
	let y_view = read_mem_addr(client.proc_handle, client.value_pointers["viewTable"] + 0x24C, 4).unwrap();

	println!("LocalPlayer -> x: {:.4}, y: {:.4} // xView: {:.4} yView: {:.4}", f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32), f32::from_bits(x_view as u32), f32::from_bits(y_view as u32));
	println!("Player count: {}", read_mem_addr(client.proc_handle, client.base_address + PLAYER_COUNT_OFFSET, 4).unwrap());
	for (_, list) in &mut client.entity_lists {
		for bot in &list.ent_vec {
			let dist = euclid_dist((f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32)), (bot.position.x, bot.position.y));
			if dist < 60.0 {
				println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} \x1b[93m({}m)\x1b[0m", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);
			}
			else {
				println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} ({}m)", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);
			}
		}
		list.refresh();
	}
	print!("{}[2J", 27 as char);
}

fn process_init(proc_name: &str) -> Result<(usize, HANDLE), String> {
	let pid = match get_process_pid(proc_name) {
		Ok(id) => id,
		Err(_) => return Err("Unable to find game process PID".to_string())
	};
	let proc_addr = match get_process_address(pid) {
		Ok(addr) => addr,
		Err(_) => return Err("Unable to find game process base address".to_string())
	};
	let proc_handle = match get_process_handle(pid) {
		Ok(handle) => handle,
		Err(_) => return Err("Unable to get handle to game process".to_string())
	};
	println!("[rust_client] Found game process and created handle with PROCESS_VM_READ");
	Ok((proc_addr, proc_handle))
}

#[cfg(test)]
mod tests {
    use crate::{process_init, memory::nop_address};

    #[test]
	fn get_handle_to_game() {
		process_init("ac_client.exe").expect("Failed to get handle to game process");
	}
	
	#[test]
	fn nop_ammo_instruction() {
		let (_, proc_addr) = process_init("ac_client.exe").expect("Cannot get handle to game process");
		nop_address(proc_addr, 0x004637E9).expect("Failed to place NOP instruction at 0x004637E9");
		nop_address(proc_addr, 0x004637EA).expect("Failed to place NOP instruction at 0x004637EA");
	}	
}
