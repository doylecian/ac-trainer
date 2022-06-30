#![allow(dead_code)]
mod memory;
mod structs;
mod math;

use core::time;
use std::thread;
use memory::{get_process_pid, get_process_address};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use crate::{memory::{get_process_handle, resolve_pointer_chain, read_mem_addr, AddressType}, math::euclid_dist};

const PROCESS_NAME: &str = "ac_client.exe";
const PLAYER_BASE: usize = 0x00109B74;
const PLAYER_HEALTH_0FFSETS: [usize; 2] = [PLAYER_BASE, 0xF8];
const PLAYER_VIEW_YAW_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0040];
const PLAYER_VIEW_PITCH_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0044];
const ENTITY_LIST_START: [usize; 1] = [0x10f4f8];
const ENTITY_COORDS_OFFSET: [usize; 3] = [0x4, 0x8, 0xc];
const ENTITY_NAME_OFFSET: usize = 0x224;

struct EntityList {
	start: usize,
	gap: i8,
	entity_count: i8,
	handle: HANDLE,
	ent_vec: Vec<Entity>
}

struct Coords {
	x: f32,
	z: f32, 
	y: f32
}

struct Entity {
	addr: usize,
	name: String,
	position: Coords
}

impl Entity {
	fn new(addr: usize, name: String) -> Self{
		Self { addr, name, position: Coords{ x: 0.0, z:  0.0, y: 0.0 } }
	}

	fn update_coords(&mut self, handle: HANDLE) {
		let x_pos = read_mem_addr(handle, self.addr + ENTITY_COORDS_OFFSET[0], 4).unwrap();
		let y_pos = read_mem_addr(handle, self.addr + ENTITY_COORDS_OFFSET[1], 4).unwrap();
		self.position.x = f32::from_bits(x_pos as u32);
		self.position.y = f32::from_bits(y_pos as u32);
	}

	fn get_coords(&self) -> f32 {
		self.position.x
	}
	
}


impl EntityList {
	fn new(start: usize, gap: i8, entity_count: i8, handle: HANDLE) -> Self{
		Self { start, gap, entity_count, handle, ent_vec: Vec::new() }
	}

	fn add_entity(&mut self) {
		let ent_ptr = self.start + ((self.gap as usize) * (self.ent_vec.len()+1));
		if let Some(addr) = read_mem_addr(self.handle, ent_ptr, 4) {
			if let Some(name) = read_mem_addr(self.handle, addr + ENTITY_NAME_OFFSET, 8) {
				let mut name_bytes = name.to_le_bytes().to_vec();
		 		name_bytes.retain(|&x| x != 0);
				let ent_name = std::str::from_utf8(&name_bytes).unwrap();
				self.ent_vec.push(Entity::new(addr, ent_name.to_string()));
				println!("Adding Entity {} found at {:X}", ent_name, addr);
			}
			
		}
	}

	fn refresh(&mut self) {
		for ent in &mut self.ent_vec {
			ent.update_coords(self.handle)
		}
	}

	// fn remove_entity(&mut self, addr: usize) {
	// 	self.entity_pointers.retain(|&x| x != addr);
	// }

	fn populate(&mut self, ents: i8) {
		while self.ent_vec.len() < ents.try_into().unwrap() {
			self.add_entity()
		}
	}
}

fn main() {
	println!("\nClient loaded successfuly! \n--------------------------\n");

	let pid = get_process_pid(PROCESS_NAME).expect("Unable to locate process");
	println!("[rust_client] Found target process {:?} ", pid);
	let proc_addr = get_process_address(pid).expect("ERROR: Couldn't get modBaseAddr of process");
	println!("[rust_client] Found target process base modBaseAddr {:?} ", proc_addr);
	let proc_handle = get_process_handle(pid).expect("ERROR: Couldn't get handle to process");
	println!("[rust_client] Created handle to process with PROCESS_VM_READ");

	let frametime = time::Duration::from_millis(10);
	if let Some(ent_list_addr) = resolve_pointer_chain(proc_handle, proc_addr, &ENTITY_LIST_START, AddressType::Pointer) {
		let mut ent_list = EntityList::new(ent_list_addr, 0x4, 5, proc_handle);
		ent_list.populate(5);
		loop {
			let local_player_xpos_addr = read_mem_addr(proc_handle, proc_addr + PLAYER_BASE, 4).unwrap();
			let local_player_xpos = read_mem_addr(proc_handle, local_player_xpos_addr + ENTITY_COORDS_OFFSET[0], 4).unwrap();
			let local_player_ypos = read_mem_addr(proc_handle, local_player_xpos_addr + ENTITY_COORDS_OFFSET[1], 4).unwrap();
			println!("LocalPlayer -> x: {:.4}, y: {:.4}", f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32));
			for bot in &ent_list.ent_vec {
				let dist = euclid_dist((f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32)), (bot.position.x, bot.position.y));
				if dist < 60.0 {
					println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} \x1b[93m({}m)\x1b[0m", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);

				}
				else {
					println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} ({}m)", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);
				}
			}
			ent_list.refresh();
			thread::sleep(frametime);
			print!("{}[2J", 27 as char);
		}
	}
	unsafe { CloseHandle(proc_handle); }
}