// Thread-safe raw tire pressures model shared between scanner and telemetry

#[derive(Default, Debug, Clone, Copy)]
pub struct SharedTirePressures {
    pub fl: f32,
    pub fr: f32,
    pub rl: f32,
    pub rr: f32,
    pub fl_temp: f32,
    pub fr_temp: f32,
    pub rl_temp: f32,
    pub rr_temp: f32,
}
