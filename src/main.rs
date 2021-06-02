mod block;
mod render;

use raylib::prelude::*;
use std::io::prelude::*;
use block::*;
use render::*;
use rand::prelude::*;
use std::fs::{OpenOptions, File};

const DELAY: f32 = 1.0 / 120.0;
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
const DROP_TIME: f32 = 0.05;
const CLEAR_TIME: f32 = 0.05;
const SIDE_MOVE_DELAY: f32 = 0.25;
const SIDE_MOVE_TIME: f32 = 0.05;

struct MainLoop {
	elapsed: f32,
	elapsed1: f32,
	elapsed2: f32,
	dir: i32,
	block: Block,
	rng: ThreadRng,
	grid: [usize; TETRIS_W * TETRIS_H],
	blocks_order: [usize; 7],
	score: u128,
	best_score: u128,
	game_over: bool,
	line_clear: usize,
	full_lines: [bool; TETRIS_H],
}
impl MainLoop {
	fn new() -> Self {
		let mut rng = thread_rng();
		let mut order = [0, 1, 2, 3, 4, 5, 6];
		order.shuffle(&mut rng);

		Self {
			elapsed: 0.0,
			elapsed1: 0.0,
			elapsed2: 0.0,
			dir: 0,
			block: Block::new(order[0]),
			rng: rng,
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
		}
	}
	fn update(&mut self, rh: &mut RaylibHandle) {
		if self.game_over {return;}

		self.elapsed1 += DELAY;
		self.elapsed += DELAY;
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
			self.elapsed = 0.0;
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
				
				if !left_col && right_col {
					self.block.pos[0] -= 2.0;
				}
				else if !left_col && !right_col {
					self.block.pos[1] -= 1.0;
				}
				else if left_col && right_col {
					if self.block.rot > 0 {self.block.rot -= 1;}
					else {self.block.rot += 3;}
					self.block.pos[0] -= 1.0;
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

			while self.block.block_collision(&self.grid) {
				self.block.pos[1] -= 1.0;
			}
			if self.block.reached_height() {
				self.game_over = true;
				return;
			}
			set_grid(&mut self.grid, &self.block);
			self.blocks_order.swap(0, 1);
			self.blocks_order.swap(1, 2);
			self.blocks_order[2..7].shuffle(&mut self.rng);
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
		}
		if self.best_score < self.score {self.best_score = self.score;}
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
			Rectangle::new(END[0] as f32 + BLOCK_SIZE as f32 - 5.0, START[1] as f32 - 5.0, (BLOCK_SIZE * 5) as f32 + 10.0, (BLOCK_SIZE * 10) as f32 + 10.0), 
			5, Color::WHITE
		);
		render_block(&{
			let mut b = Block::new(self.blocks_order[1]);
			b.pos[0] += 9.0;
			b.pos[1] += 1.0;
			b
		}, rdh, 255, true, [
			if self.blocks_order[1] == 0 || self.blocks_order[1] == 3 {-BLOCK_SIZE / 2}
			else {0},
			0
		]);
		render_block(&{
			let mut b = Block::new(self.blocks_order[2]);
			b.pos[0] += 9.0;
			b.pos[1] += 6.0;
			b
		}, rdh, 255, true, [
			if self.blocks_order[2] == 0 || self.blocks_order[2] == 3 {-BLOCK_SIZE / 2}
			else {0},
			0
		]);
		if !self.game_over {rdh.draw_text(format!("Score: {}", self.score).as_str(), START[0], START[1] - 40, 20, Color::WHITE);}
		else {
			const FONT_SIZE: i32 = 20;
			rdh.draw_rectangle(0, 0, WIDTH, HEIGHT, Color::new(0, 0, 0, (256 / 4 * 3) as u8));
			rdh.draw_text(format!("Game over!\nLast score: {}\nBest score: {}", self.score, self.best_score).as_str(),
				-FONT_SIZE * 3 + WIDTH / 2, -FONT_SIZE * 2 + HEIGHT / 2, FONT_SIZE, Color::WHITE);
		}
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

	let mut file = OpenOptions::new().write(true).truncate(true).open(SAVE_PATH).unwrap();
	write!(file, "{}", main_loop.best_score).unwrap();
}
