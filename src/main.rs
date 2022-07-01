#![allow(dead_code)]
mod memory;
mod structs;
mod math;

use core::time;
use std::{self, thread};
use memory::{get_process_pid, get_process_address};
use windows::Win32::{Foundation::{CloseHandle, HANDLE}, UI::Input::KeyboardAndMouse::GetAsyncKeyState};
use crate::{memory::{get_process_handle, resolve_pointer_chain, read_mem_addr, AddressType}, math::euclid_dist};
use std::collections::HashMap;

const PROCESS_NAME: &str = "ac_client.exe";
const PLAYER_BASE: usize = 0x00109B74;
const PLAYER_HEALTH_0FFSETS: [usize; 2] = [PLAYER_BASE, 0xF8];
const PLAYER_VIEW_YAW_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0040];
const PLAYER_VIEW_PITCH_OFFSETS: [usize; 2] = [PLAYER_BASE, 0x0044];
const ENTITY_LIST_START: [usize; 1] = [0x10f4f8];
const ENTITY_COORDS_OFFSET: [usize; 3] = [0x4, 0x8, 0xc];
const ENTITY_NAME_OFFSET: usize = 0x224;
const PLAYER_COUNT_OFFSET: usize = 0x10F500;

struct Game {
	proc_handle: HANDLE,
	base_address: usize,
	offsets: HashMap<String, Vec<usize>>,
	entity_lists: HashMap<String, EntityList>
}

impl Game {
	fn new(proc_handle: HANDLE, base_address: usize) -> Self {
		Self { proc_handle, base_address, offsets: HashMap::new(), entity_lists: HashMap::new() }
	}

	fn add_offset(&mut self, offsets: (String, Vec<usize>)) {
		self.offsets.insert(offsets.0, offsets.1);
	}

	fn show_entity_lists(&self) {
		for (k, v) in &self.entity_lists {
			println!("[ENTITYLIST] {} - > {:X}", k, v.start)
		}
	}

	fn add_entity_list(&mut self, name: &str, ent_list: EntityList) {
		self.entity_lists.insert(name.to_string(), ent_list);
	}

	fn update_entity_list_addr(&mut self, name: &str) {
		let current_list = &self.entity_lists[name];
		let mut ent_list = EntityList::new(current_list.start, current_list.gap, current_list.entity_count, current_list.handle);
		ent_list.populate(current_list.entity_count);
		self.entity_lists.remove(name);
		self.entity_lists.insert(name.to_string(), ent_list);
	}
}

#[derive(Clone)]
struct EntityList {
	start: usize,
	gap: u8,
	entity_count: u8,
	handle: HANDLE,
	ent_vec: Vec<Entity>
}

#[derive(Copy, Clone)]
struct Coords {
	x: f32,
	z: f32, 
	y: f32
}

#[derive(Clone)]
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

impl std::fmt::Debug for EntityList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let start = format!("0x{:X}", &self.start);
        f.debug_struct("EntityList")
		.field("Memory address", &start)
		.field("Entity count", &self.entity_count)
		.field("Distance between entities", &self.gap)
         .field("Entity vector", &self.ent_vec)
         .finish()
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let addr = format!("0x{:X}", &self.addr);
        f.debug_struct("Entity")
		.field("Memory address", &addr)
		.field("Entity name", &self.name)
        .finish()
    }
}

impl EntityList {
	fn new(start: usize, gap: u8, entity_count: u8, handle: HANDLE) -> Self{
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

	fn remove_entity(&mut self, addr: usize) {
		self.ent_vec.retain(|x| x.addr != addr);
	}

	fn populate(&mut self, ents: u8) {
		while self.ent_vec.len() < ents.try_into().unwrap() {
			self.add_entity()
		}
	}
}

fn main() {

	let (game_handle, proc_addr) = process_init("ac_client.exe").expect("Cannot initialise game hack - make sure the game is running");
	let mut client = Game::new(proc_addr, game_handle);
	let frametime = time::Duration::from_millis(10);
	if let Some(ent_list_addr) = resolve_pointer_chain(client.proc_handle, client.base_address, &ENTITY_LIST_START, AddressType::Pointer) {
		let mut ent_list = EntityList::new(ent_list_addr, 0x4, 5, client.proc_handle);
		ent_list.populate(5);
		client.add_entity_list("Bot player list", ent_list);
	}
	loop {
		update(&mut client);
		thread::sleep(frametime);
		unsafe {
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
	println!("LocalPlayer -> x: {:.4}, y: {:.4}", f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32));
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

	// for bot in &ent_list.ent_vec {
			// 	let dist = euclid_dist((f32::from_bits(local_player_xpos as u32), f32::from_bits(local_player_ypos as u32)), (bot.position.x, bot.position.y));
			// 	if dist < 60.0 {
			// 		println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} \x1b[93m({}m)\x1b[0m", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);

			// 	}
			// 	else {
			// 		println!("BOT {:?} -> {{x: {:.4}, y: {:.4}}} ({}m)", bot.name, bot.position.x, bot.position.y, dist as u32 / 10);
			// 	}
			// }
			//ent_list.refresh();


	