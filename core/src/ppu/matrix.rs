use serde::{Deserialize, Serialize};

pub fn convert_8p8(value: u16) -> f32 {
    // ((value as i16 >> 8) as f32) + (value & 0xFF) as i8 as f32 / 0x100 as f32
    (value as i16 as f32) / 0x100 as f32
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Matrix {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub center_x: i16,
    pub center_y: i16,
}

impl Matrix {
    /// Multiply a vector by the matrix
    pub fn multiply(&self, lhs: [f32; 2]) -> [f32; 2] {
        let a = [
            lhs[0] as f32 - self.center_x as f32,
            lhs[1] as f32 - self.center_y as f32,
            1.0,
        ];
        let mat = [
            [
                convert_8p8(self.a),
                convert_8p8(self.b),
                self.center_x as f32,
            ],
            [
                convert_8p8(self.c),
                convert_8p8(self.d),
                self.center_y as f32,
            ],
            [0.0, 0.0, 1.0],
        ];
        let res: [f32; 2] =
            core::array::from_fn(|i| a.iter().enumerate().map(|(j, v)| mat[i][j] * *v).sum());
        res
    }
}
