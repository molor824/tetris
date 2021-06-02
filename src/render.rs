use crate::*;

pub fn render_grid(grid: &[usize], d: &mut RaylibDrawHandle) {
	for y in 0..TETRIS_H {
		for x in 0..TETRIS_W {
			let v = grid[x + y * TETRIS_W];
			d.draw_rectangle(
				START[0] + x as i32 * BLOCK_SIZE,
				START[1] + y as i32 * BLOCK_SIZE,
				BLOCK_SIZE, BLOCK_SIZE, if v != 0 {let col = BLOCK_COLS[v - 1]; Color::new(col[0], col[1], col[2], 255)}
					else {Color::new(0, 0, 0, 0)}
			);
		}
	}
}
pub fn render_block(b: &Block, d: &mut RaylibDrawHandle, transparency: u8) {
	for i in 0..4 {
		let mut offset = BLOCK_OFFSETS[b.block_type][i].rotate90(b.rot);
		offset[0] += b.pos[0]; offset[1] += b.pos[1];
		offset[0] = offset[0].round(); offset[1] = offset[1].round();
		if (offset[0] as i32) < 0 || (offset[0] as i32) >= TETRIS_W as i32 || 
			(offset[1] as i32) < 0 || (offset[1] as i32) >= TETRIS_H as i32 {continue;}

		let col = BLOCK_COLS[b.block_type];
		d.draw_rectangle(
			START[0] + offset[0] as i32 * BLOCK_SIZE,
			START[1] + offset[1] as i32 * BLOCK_SIZE,
			BLOCK_SIZE, BLOCK_SIZE, Color::new(col[0], col[1], col[2], transparency)
		);
	}
}
pub fn render_grid_outline(d: &mut RaylibDrawHandle) {
	for y in 0..TETRIS_H {
		for x in 0..TETRIS_W {
			d.draw_rectangle_lines_ex(Rectangle::new((START[0] + x as i32 * BLOCK_SIZE) as f32,
				(START[1] + y as i32 * BLOCK_SIZE) as f32,
				BLOCK_SIZE as f32, BLOCK_SIZE as f32), GRID_OUTLINE_THICKNESS, Color::BLACK);
		}
	}
}
pub fn set_grid(grid: &mut [usize], b: &Block) {
	for i in 0..4 {
		let mut offset = BLOCK_OFFSETS[b.block_type][i].rotate90(b.rot);
		offset[0] += b.pos[0]; offset[1] += b.pos[1];
		offset[0] = offset[0].round(); offset[1] = offset[1].round();

		let idx = offset[0] as i32 + offset[1] as i32 * TETRIS_W as i32;
		if idx < 0 || idx >= grid.len() as i32 {continue;}

		grid[idx as usize] = b.block_type + 1;
	}
}