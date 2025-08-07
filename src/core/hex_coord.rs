// Hexagonal coordinate system implementation

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    
    pub fn distance(&self, other: &HexCoord) -> i32 {
        ((self.q - other.q).abs() + (self.q + self.r - other.q - other.r).abs() + (self.r - other.r).abs()) / 2
    }
    
    pub fn neighbors(&self) -> Vec<HexCoord> {
        // Original axial coordinate neighbors (used for map generation)
        vec![
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1),
            HexCoord::new(self.q, self.r + 1),
        ]
    }
    
    pub fn offset_neighbors(&self) -> Vec<HexCoord> {
        // For offset coordinates with pointy-top hexagons (used for movement)
        // Odd columns (q % 2 == 1) are shifted down
        if self.q % 2 == 0 {
            // Even column
            vec![
                HexCoord::new(self.q + 1, self.r - 1), // NE
                HexCoord::new(self.q + 1, self.r),     // SE
                HexCoord::new(self.q, self.r + 1),     // S
                HexCoord::new(self.q - 1, self.r),     // SW
                HexCoord::new(self.q - 1, self.r - 1), // NW
                HexCoord::new(self.q, self.r - 1),     // N
            ]
        } else {
            // Odd column (shifted down)
            vec![
                HexCoord::new(self.q + 1, self.r),     // NE
                HexCoord::new(self.q + 1, self.r + 1), // SE
                HexCoord::new(self.q, self.r + 1),     // S
                HexCoord::new(self.q - 1, self.r + 1), // SW
                HexCoord::new(self.q - 1, self.r),     // NW
                HexCoord::new(self.q, self.r - 1),     // N
            ]
        }
    }
} 