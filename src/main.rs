mod block;
mod render;

use raylib::prelude::*;
use std::io::prelude::*;
use block::*;
use render::*;
use std::fs::{OpenOptions, File};
use std::io::Write as ioWrite;
use std::fmt::Write as fmtWrite;

const DELAY: f32 = 1.0 / 100.0;
const GRID_OUTLINE_THICKNESS: i32 = 1;
const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const TETRIS_W: usize = 10;
const TETRIS_H: usize = 24;
const BLOCK_SIZE: i32 = 20;
const START: [i32; 2] = [WIDTH / 2 - TETRIS_W as i32 * BLOCK_SIZE / 2, HEIGHT / 2 - TETRIS_H as i32 * BLOCK_SIZE / 2];
const END: [i32; 2] = [WIDTH / 2 + TETRIS_W as i32 * BLOCK_SIZE / 2, HEIGHT / 2 + TETRIS_H as i32 * BLOCK_SIZE / 2];
const SAVE_PATH: &str = "best_score";
const FALL_TIME: f32 = 1.5;
const DROP_TIME: f32 = 0.025;
const CLEAR_TIME: f32 = 0.025;
const SIDE_MOVE_DELAY: f32 = 0.2;
const SIDE_MOVE_TIME: f32 = 0.05;
const FONT_SIZE: i32 = 20;

trait Shuffle {
	fn shuffle(&mut self);
}
impl Shuffle for [usize] {
	fn shuffle(&mut self) {
		let len = self.len();

		for i in 0..len {
			self.swap(i, get_random_value::<i32>(0, len as i32 - 1) as usize);
		}
	}
}
struct MainLoop {
	elapsed: f32,
	elapsed1: f32,
	elapsed2: f32,
	dir: i8,
	block: Block,
	grid: [usize; TETRIS_W * TETRIS_H],
	blocks_order: [usize; 7],
	score: u128,
	best_score: u128,
	game_over: bool,
	line_clear: usize,
	full_lines: [bool; TETRIS_H],
	hold_block_type: usize,
	hold_block: bool,
	txt: String,
}
impl MainLoop {
	fn new() -> Self {
		let mut order = [0, 1, 2, 3, 4, 5, 6];
		order.shuffle();

		Self {
			elapsed: 0.0,
			elapsed1: 0.0,
			elapsed2: 0.0,
			dir: 0,
			block: Block::new(order[0]),
			grid: [0; TETRIS_W * TETRIS_H],
			blocks_order: order,
			score: 0,
			best_score: {
				let mut text = String::new();
				if let Ok(mut f) = File::open(SAVE_PATH) {
					if let Ok(_) = f.read_to_string(&mut text) {
						if let Ok(n) = text.parse() {n}
						else {0}
					}
					else {0}
				}
				else {
					File::create(SAVE_PATH).unwrap();
					0
				}
			},
			game_over: false,
			line_clear: 0,
			full_lines: [false; TETRIS_H],
			hold_block_type: 0,
			hold_block: false,
			txt: String::new(),
		}
	}
	fn update(&mut self, rh: &mut RaylibHandle) {
		if self.game_over {
			if rh.is_key_pressed(KeyboardKey::KEY_ENTER) {
				let temp = self.best_score;
				*self = Self::new();
				self.best_score = temp;
			}
			return;
		}

		self.elapsed1 += DELAY;
		if self.dir != 0 {self.elapsed2 -= DELAY;}
		
		if self.line_clear != 0 {
			if self.elapsed1 >= CLEAR_TIME {
				self.elapsed1 = 0.0;
				self.line_clear -= 1;
				for i in 0..TETRIS_H {
					if self.full_lines[i] {
						self.grid[self.line_clear + i * TETRIS_W] = 0;
						self.grid[TETRIS_W - self.line_clear - 1 + i * TETRIS_W] = 0;
					}
				}
			}
			return;
		}
		for i in 0..TETRIS_H {
			if self.full_lines[i] {
				for y in (0..=i).rev() {
					for x in 0..TETRIS_W {
						let idx = x + y * TETRIS_W;
						self.grid[idx] = if y == 0 {0} else {self.grid[idx - TETRIS_W]};
					}
				}
				self.full_lines[i] = false;
			}
		}
		self.elapsed += DELAY;
		let hold_block_type = self.hold_block_type;
		if rh.is_key_pressed(KeyboardKey::KEY_C) && !self.hold_block {
			self.hold_block = true;
			self.hold_block_type = self.block.block_type + 1;
			if hold_block_type == 0 {
				self.blocks_order.swap(0, 1);
				self.blocks_order.swap(1, 2);
				self.blocks_order[2..7].shuffle();
				self.block = Block::new(self.blocks_order[0]);
	
				while self.block.block_collision(&self.grid) {
					self.block.pos[1] -= 1.0;
				}
			}
			else {
				self.block = Block::new(hold_block_type - 1);
				self.blocks_order[0] = hold_block_type - 1;
			}
		}
		if rh.is_key_pressed(KeyboardKey::KEY_LEFT) {
			self.block.pos[0] -= 1.0;
			if self.block.block_collision(&self.grid) {self.block.pos[0] += 1.0;}
			self.dir = -1;
			self.elapsed2 = SIDE_MOVE_DELAY;
		}
		if rh.is_key_pressed(KeyboardKey::KEY_RIGHT) {
			self.block.pos[0] += 1.0;
			if self.block.block_collision(&self.grid) {self.block.pos[0] -= 1.0;}
			self.dir = 1;
			self.elapsed2 = SIDE_MOVE_DELAY;
		}
		if (rh.is_key_released(KeyboardKey::KEY_LEFT) && self.dir == -1) ||
			(rh.is_key_released(KeyboardKey::KEY_RIGHT) && self.dir == 1) {self.dir = 0;}

		if rh.is_key_pressed(KeyboardKey::KEY_UP) && self.block.block_type != 3 {
			self.block.rot += 1;
			if self.block.rot > 3 {self.block.rot = 0;}
			if self.block.block_collision(&self.grid) {
				self.block.pos[0] -= 1.0;
				let left_col = self.block.block_collision(&self.grid);
				self.block.pos[0] += 2.0;
				let right_col = self.block.block_collision(&self.grid);
				
				if left_col && !right_col {}
				else if !left_col && right_col {
					self.block.pos[0] -= 2.0;
				}
				else if !left_col && !right_col {
					self.block.pos[1] -= 1.0;
					if !self.block.block_collision(&self.grid) {self.block.pos[1] += 1.0;}
				}
				else {
					self.block.rot -= 1;
					if self.block.rot < 0 {self.block.rot = 3;}
				}
			}
		}
		if rh.is_key_pressed(KeyboardKey::KEY_SPACE) {
			while !self.block.block_collision(&self.grid) {
				self.block.pos[1] += 1.0;
			}
		}
		if self.elapsed2 <= 0.0 {
			self.elapsed2 = SIDE_MOVE_TIME;
			if rh.is_key_down(KeyboardKey::KEY_LEFT) && self.dir == -1 {
				self.block.pos[0] -= 1.0;
				if self.block.block_collision(&self.grid) {self.block.pos[0] += 1.0;}
			}
			else if rh.is_key_down(KeyboardKey::KEY_RIGHT) && self.dir == 1 {
				self.block.pos[0] += 1.0;
				if self.block.block_collision(&self.grid) {self.block.pos[0] -= 1.0;}
			}
		}
		if self.elapsed1 >= DROP_TIME {
			self.elapsed1 = 0.0;
			if rh.is_key_down(KeyboardKey::KEY_DOWN) {
				let temp = self.elapsed;
				self.elapsed = 0.0;
				self.block.pos[1] += 1.0;
				if self.block.block_collision(&self.grid) {
					self.block.pos[1] -= 1.0;
					self.elapsed = temp;
				}
			}
		}
		if self.elapsed >= FALL_TIME {
			self.elapsed = 0.0;
			self.block.pos[1] += 1.0;
		}
		if self.block.block_collision(&self.grid) {
			self.elapsed = 0.0;
			self.elapsed1 = 0.0;
			self.dir = 0;
			self.hold_block = false;

			while self.block.block_collision(&self.grid) {
				self.block.pos[1] -= 1.0;
			}
			if self.block.reached_height() {
				self.game_over = true;
				return;
			}
			set_grid(&mut self.grid, &self.block);
			for i in 1..7 {
				if self.blocks_order[i] == self.blocks_order[0] {
					for v in 0..7 {
						let mut matching = false;
						for c in 1..7 {
							if self.blocks_order[c] == v {matching = true; break;}
						}
						if !matching {self.blocks_order[0] = v; break;}
					}
					break;
				}
			}
			self.blocks_order.swap(0, 1);
			self.blocks_order.swap(1, 2);
			self.blocks_order[2..7].shuffle();
			self.block = Block::new(self.blocks_order[0]);

			while self.block.block_collision(&self.grid) {
				self.block.pos[1] -= 1.0;
			}

			for y in 0..TETRIS_H {
				let mut line = true;
				for x in 0..TETRIS_W {
					if self.grid[x + y * TETRIS_W] == 0 {line = false; break;}
				}
				if !line {continue;}
				self.full_lines[y] = true;
				self.line_clear = (TETRIS_W + 1) / 2;
				self.score += 10;
			}
			self.score += 1;
			if self.score > self.best_score {self.best_score = self.score;}
		}
	}
	fn render(&mut self, rdh: &mut RaylibDrawHandle) {
		rdh.clear_background(Color::BLACK);
		rdh.draw_rectangle(START[0] - 5, START[1] - 5, TETRIS_W as i32 * BLOCK_SIZE + 10, TETRIS_H as i32 * BLOCK_SIZE + 10, Color::WHITE);
		render_grid(&self.grid, rdh);

		if self.line_clear == 0 {
			let temp = self.block.pos[1];
			while !self.block.block_collision(&self.grid) {
				self.block.pos[1] += 1.0;
			}
			self.block.pos[1] -= 1.0;
			render_block(&self.block, rdh, 127, false, [0, 0]);
			self.block.pos[1] = temp;
			render_block(&self.block, rdh, 255, false, [0, 0]);
		}
		render_grid_outline(rdh);
		rdh.draw_rectangle_lines_ex(
			Rectangle::new(END[0] as f32 + (BLOCK_SIZE * 2) as f32 - 5.0, START[1] as f32 - 5.0, (BLOCK_SIZE * 5) as f32 + 10.0, (BLOCK_SIZE * 10) as f32 + 10.0), 
			5, Color::WHITE
		);
		rdh.draw_rectangle_lines_ex(
			Rectangle::new(START[0] as f32 - (BLOCK_SIZE * 2) as f32 - 5.0 - ((BLOCK_SIZE * 5) as f32 + 10.0), START[1] as f32 - 5.0, (BLOCK_SIZE * 5) as f32 + 10.0, (BLOCK_SIZE * 5) as f32 + 10.0),
			5, Color::WHITE
		);
		if self.hold_block_type != 0 {
			render_block(&{
				let mut b = Block::new(self.hold_block_type - 1);
				b.pos[0] -= 10.0;
				b.pos[1] += 1.0;
				b
			}, rdh, 255, true, [
				if self.hold_block_type == 1 || self.hold_block_type == 4 {0}
				else {BLOCK_SIZE / 2},
				BLOCK_SIZE / 2
			]);
		}
		render_block(&{
			let mut b = Block::new(self.blocks_order[1]);
			b.pos[0] += 10.0;
			b.pos[1] += 1.0;
			b
		}, rdh, 255, true, [
			if self.blocks_order[1] == 0 || self.blocks_order[1] == 3 {-BLOCK_SIZE / 2}
			else {0},
			BLOCK_SIZE / 2
		]);
		render_block(&{
			let mut b = Block::new(self.blocks_order[2]);
			b.pos[0] += 10.0;
			b.pos[1] += 6.0;
			b
		}, rdh, 255, true, [
			if self.blocks_order[2] == 0 || self.blocks_order[2] == 3 {-BLOCK_SIZE / 2}
			else {0},
			BLOCK_SIZE / 2
		]);

		self.txt.clear();
		write!(&mut self.txt, "Score: {}\nBest Score: {}", self.score, self.best_score).unwrap();

		rdh.draw_text(self.txt.as_str(), START[0], START[1] - FONT_SIZE * 3, FONT_SIZE, Color::WHITE);
		rdh.draw_text("Hold", START[0] - (BLOCK_SIZE * 2) - BLOCK_SIZE * 5 - 10, START[1] - FONT_SIZE * 2, FONT_SIZE, Color::WHITE);
		rdh.draw_text("Incoming", END[0] + (BLOCK_SIZE * 2), START[1] - FONT_SIZE * 2, FONT_SIZE, Color::WHITE);
		if self.game_over {
			self.txt.clear();
			write!(&mut self.txt, "Game Over!\nLast Score: {}\nBest Score: {}\nPress 'Enter' to restart!", self.score, self.best_score).unwrap();

			rdh.draw_rectangle(0, 0, WIDTH, HEIGHT, Color::new(0, 0, 0, (256 / 4 * 3) as u8));
			rdh.draw_text(
				self.txt.as_str(),
				-FONT_SIZE * 3 + WIDTH / 2, -FONT_SIZE * 2 + HEIGHT / 2, FONT_SIZE, Color::WHITE
			);
		}
	}
}
impl Drop for MainLoop {
	fn drop(&mut self) {
		write!(
			&mut OpenOptions::new().write(true).truncate(true).create(true).open(SAVE_PATH).unwrap(),
			"{}", self.best_score
		).unwrap();
	}
}
fn main() {
	let (mut rh, thread) = raylib::init().size(WIDTH, HEIGHT).title("tetris").build();
	rh.set_target_fps((1.0 / DELAY).round() as u32);
	
	let mut main_loop = MainLoop::new();

	while !rh.window_should_close() {
		main_loop.update(&mut rh);
		main_loop.render(&mut rh.begin_drawing(&thread));
	}
}
