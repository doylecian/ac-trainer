use std::collections::HashMap;

use windows::Win32::Foundation::HANDLE;

use crate::{memory::read_mem_addr, ENTITY_COORDS_OFFSET, ENTITY_NAME_OFFSET};

pub struct Game {
	pub proc_handle: HANDLE,
	pub base_address: usize,
	pub offsets: HashMap<String, Vec<usize>>,
	pub entity_lists: HashMap<String, EntityList>
}

#[derive(Clone)]
pub struct EntityList {
	pub start: usize,
	pub gap: u8,
	pub entity_count: u8,
	pub handle: HANDLE,
	pub ent_vec: Vec<Entity>
}

#[derive(Copy, Clone)]
pub struct Coords {
	pub x: f32,
	pub z: f32, 
	pub y: f32
}

#[derive(Clone)]
pub struct Entity {
	pub addr: usize,
	pub name: String,
	pub position: Coords
}

impl Game {
	pub fn new(proc_handle: HANDLE, base_address: usize) -> Self {
		Self { proc_handle, base_address, offsets: HashMap::new(), entity_lists: HashMap::new() }
	}

	pub fn add_offset(&mut self, offsets: (String, Vec<usize>)) {
		self.offsets.insert(offsets.0, offsets.1);
	}

	pub fn show_entity_lists(&self) {
		for (k, v) in &self.entity_lists {
			println!("[ENTITYLIST] {} - > {:X}", k, v.start)
		}
	}

	pub fn add_entity_list(&mut self, name: &str, ent_list: EntityList) {
		self.entity_lists.insert(name.to_string(), ent_list);
	}

	pub fn update_entity_list_addr(&mut self, name: &str) {
		let current_list = &self.entity_lists[name];
		let mut ent_list = EntityList::new(current_list.start, current_list.gap, current_list.entity_count, current_list.handle);
		ent_list.populate(current_list.entity_count);
		self.entity_lists.remove(name);
		self.entity_lists.insert(name.to_string(), ent_list);
	}
}


impl Entity {
	pub fn new(addr: usize, name: String) -> Self{
		Self { addr, name, position: Coords{ x: 0.0, z:  0.0, y: 0.0 } }
	}

	pub fn update_coords(&mut self, handle: HANDLE) {
		let x_pos = read_mem_addr(handle, self.addr + ENTITY_COORDS_OFFSET[0], 4).unwrap();
		let y_pos = read_mem_addr(handle, self.addr + ENTITY_COORDS_OFFSET[1], 4).unwrap();
		self.position.x = f32::from_bits(x_pos as u32);
		self.position.y = f32::from_bits(y_pos as u32);
	}

	pub fn get_coords(&self) -> f32 {
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
	pub fn new(start: usize, gap: u8, entity_count: u8, handle: HANDLE) -> Self{
		Self { start, gap, entity_count, handle, ent_vec: Vec::new() }
	}

	pub fn add_entity(&mut self) {
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

	pub fn refresh(&mut self) {
		for ent in &mut self.ent_vec {
			ent.update_coords(self.handle)
		}
	}

	pub fn remove_entity(&mut self, addr: usize) {
		self.ent_vec.retain(|x| x.addr != addr);
	}

	pub fn populate(&mut self, ents: u8) {
		while self.ent_vec.len() < ents.try_into().unwrap() {
			self.add_entity()
		}
	}
}
