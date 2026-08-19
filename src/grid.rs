use bevy::prelude::{Resource, Transform};
use std::f32::consts::{FRAC_PI_6};
use bevy::math::Quat;
use rand::RngExt;

#[derive(Resource)]
pub struct TransformGrid {
    rows: usize,
    cols: usize,
    y: f32,
    transforms: Vec<Transform>,
    next: usize,
}

impl TransformGrid {
    pub fn new(rows: usize, cols: usize, y: f32) -> Self {
        let mut tg = Self{rows, cols, transforms: Vec::new(), next: 0, y};
        let rs = tg.generate_rectangular_spiral();
        let gs = tg.generate_grid_sequence(rs);
        tg.generate_grid_transforms(gs);
        tg
    }

    pub fn next(&mut self) -> Transform {
        if self.next >= self.transforms.len() {
            self.next = 0;
        }
        let t = self.transforms[self.next];
        self.next += 1;
        t
    }

    fn generate_rectangular_spiral(&mut self) -> Vec<Vec<usize>> {
        let mut grid = vec![vec![0; self.cols]; self.rows];
        let total_cells = self.rows * self.cols;

        // Find the center coordinates
        let mut y = (self.rows / 2) as isize;
        let mut x = (self.cols / 2) as isize;

        // Direction vectors: Right, Down, Left, Up
        let dy = [0, 1, 0, -1];
        let dx = [1, 0, -1, 0];

        let mut current_dir = 0; // Start by moving Right
        let mut step_length = 1;
        let mut val = 0;

        // Place the first value at the center if grid exists
        if self.rows > 0 && self.cols > 0 {
            grid[y as usize][x as usize] = val;
            val += 1;
        }

        // Keep looping until every single cell is filled
        while val < total_cells {
            for _ in 0..2 {
                for _ in 0..step_length {
                    y += dy[current_dir];
                    x += dx[current_dir];

                    // Strictly check rectangular boundaries
                    if y >= 0 && y < self.rows as isize && x >= 0 && x < self.cols as isize {
                        grid[y as usize][x as usize] = val;
                        val += 1;
                    }
                }
                // Turn clockwise
                current_dir = (current_dir + 1) % 4;
            }
            // Increment step distance after two turns
            step_length += 1;
        }
        grid
    }
    fn generate_grid_sequence( &self, grid: Vec<Vec<usize>>) -> (Vec<f32>, Vec<f32>){
        let rows = self.rows;
        let cols = self.cols;
        let mut r: Vec<f32> = vec![0.0; rows*cols];
        let mut c: Vec<f32> = vec![0.0; rows*cols];
        // Print the grid formatted
        let mut rx: i32 = -(rows as i32/2);
        let mut cx: i32 = -(cols as i32/2);
        for row in grid {
            for num in row {
                r[num] = rx as f32;
                c[num] = cx as f32;
                cx += 1;
            }
            rx += 1;
            cx = -(cols as i32/2);
        }
        (r,c)
    }

    fn generate_grid_transforms(&mut self, xz: (Vec<f32>,Vec<f32>)) {
        let mut rng = rand::rng();
        for i in 0..xz.0.len() {
            let r = rng.random_range(-FRAC_PI_6..FRAC_PI_6);
            self.transforms.push(
                Transform::from_xyz(xz.0[i]*4.0, self.y, xz.1[i]*4.0)
                    .with_rotation(Quat::from_rotation_z(r))
            );
        }
    }
}
