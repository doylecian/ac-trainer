#![allow(dead_code)]
mod memory;
mod structs;
mod math;

use core::time;
use std::{self, thread};
use memory::{get_process_pid, get_process_address, write_float};
use structs::{Game, EntityList};
use windows::Win32::{Foundation::{CloseHandle, HANDLE}, UI::Input::KeyboardAndMouse::GetAsyncKeyState};
use crate::{memory::{get_process_handle, resolve_pointer_chain, read_mem_addr, AddressType}, math::euclid_dist};

const PROCESS_NAME: &str = "ac_client.exe";
const PLAYER_BASE: usize = 0x00109B74;
const PLAYER_HEALTH_0FFSETS: [usize; 2] = [PLAYER_BASE, 0xF8];
const PLAYER_VIEW_YAW_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0040];
const PLAYER_VIEW_PITCH_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0044];
const ENTITY_LIST_START: [usize; 1] = [0x10f4f8];
const ENTITY_COORDS_OFFSET: [usize; 3] = [0x4, 0x8, 0xc];
const ENTITY_NAME_OFFSET: usize = 0x224;
const PLAYER_COUNT_OFFSET: usize = 0x10F500;
const Y_VIEW_OFFSETS: [usize; 5] = [0x0010A280, 0x8, 0x214, 0x70, 0x6c];

fn main() {

	let (game_handle, proc_addr) = process_init("ac_client.exe").expect("Cannot initialise game hack - make sure the game is running");
	let mut client = Game::new(proc_addr, game_handle);

	if let Some(v) = resolve_pointer_chain(client.proc_handle, client.base_address, &Y_VIEW_OFFSETS, AddressType::Pointer) {
		client.add_address("viewTable".to_string(), v);
	}

	if let Some(ent_list_addr) = resolve_pointer_chain(client.proc_handle, client.base_address, &ENTITY_LIST_START, AddressType::Pointer) {
		let mut ent_list = EntityList::new(ent_list_addr, 0x4, 5, client.proc_handle);
		ent_list.populate(5);
		client.add_entity_list("Bot player list", ent_list);
	}

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
				CloseHandle(client.proc_handle);
				return;
			}
		}
	}
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

	