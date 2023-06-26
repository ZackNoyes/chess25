
pub struct Weights {
    pub pieces: [[f32; 6]; 2],
    pub mobility: [f32; 2],
    pub king_safety: [f32; 2],
    pub pawn_advancement: [f32; 2],
}

pub struct FeatureEval {

}