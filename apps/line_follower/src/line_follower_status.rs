use crate::board;

// Line follower state shared between the different states
pub struct LineFollowerStatus {
    pub board: board::Mightybuga_BSC,
    pub light_sensor_thresholds: Option<[u16; 8]>,
}
