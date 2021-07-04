use crate::*;

pub trait Rotate90 {
	fn rotate90(&self, rot: i8) -> Self;
}
impl Rotate90 for [f32; 2] {
	fn rotate90(&self, rot: i8) -> Self {
		match rot {
			1 => [-self[1], self[0]],
			2 => [-self[0], -self[1]],
			3 => [self[1], -self[0]],
			_ => *self
		}
	}
}
pub const BLOCK_OFFSETS: [[[f32; 2]; 4]; 7] = [
	[[-1.5, -0.5], [-0.5, -0.5], [0.5, -0.5], [1.5, -0.5]], //I
	[[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0], [1.0, -1.0]], //J
	[[-1.0, -1.0], [-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]], //L
	[[-0.5, -0.5], [0.5, -0.5], [-0.5, 0.5], [0.5, 0.5]], //O
	[[-1.0, 0.0], [0.0, 0.0], [0.0, -1.0], [1.0, -1.0]], //S
	[[-1.0, 0.0], [0.0, 0.0], [0.0, -1.0], [1.0, 0.0]], //T
	[[-1.0, -1.0], [0.0, -1.0], [0.0, 0.0], [1.0, 0.0]], //Z
];
pub const DEFAULT_POS: [[f32; 2]; 2] = [[4.0, 1.0], [4.5, 0.5]];
pub const BLOCK_COLS: [[u8; 3]; 7] = [[0, 255, 255], [0, 0, 255], [255, 127, 0], [255, 255, 0], [0, 255, 0], [255, 0, 255], [255, 0, 0]];
pub struct Block {
	pub pos: [f32; 2],
	pub rot: i8,
	pub block_type: usize,
}
impl Block {
	pub fn new(block_type: usize) -> Self {
		Self {
			pos: DEFAULT_POS[if block_type == 0 || block_type == 3 {1} else {0}],
			rot: 0,
			block_type: block_type,
		}
	}
	pub fn block_collision(&self, grid: &[usize]) -> bool {
		for i in 0..4 {
			let mut offset = BLOCK_OFFSETS[self.block_type][i].rotate90(self.rot);
			offset[0] += self.pos[0]; offset[1] += self.pos[1];
			offset[0] = offset[0].round(); offset[1] = offset[1].round();
			
			let idx = offset[0] as i32 + offset[1] as i32 * TETRIS_W as i32;
			if idx < grid.len() as i32 && idx >= 0 && grid[idx as usize] != 0 ||
				(offset[0] as i32) < 0 || offset[0] as i32 >= TETRIS_W as i32 || (offset[1] as i32) >= TETRIS_H as i32
			{
				return true;
			}
		}
		false
	}
	pub fn reached_height(&self) -> bool {
		for i in 0..4 {
			let h = BLOCK_OFFSETS[self.block_type][i].rotate90(self.rot)[1] + self.pos[1];
			if h.round() < 0.0 {return true;}
		}
		false
	}
}