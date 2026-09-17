use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use log::{info, warn};

static SYSTEM_HEALTH: Mutex<CriticalSectionRawMutex, SystemHealth> =
    Mutex::new(SystemHealth::new());
#[derive(Default, Clone)]
struct SystemHealth {
    imu_is_up: bool,
    baro_is_up: bool,
}

impl SystemHealth {
    const fn new() -> Self {
        Self {
            imu_is_up: false,
            baro_is_up: false,
        }
    }
}
pub async fn log_health() {
    let health = SYSTEM_HEALTH.lock().await.clone();
    if !health.imu_is_up || !health.baro_is_up {
        warn!(
            "imu_is_up: {}, baro_is_up: {}",
            health.imu_is_up, health.baro_is_up
        );
    } else {
        info!(
            "imu_is_up: {}, baro_is_up: {}",
            health.imu_is_up, health.baro_is_up
        );
    }
}
pub async fn update_imu_health(is_up: bool) {
    SYSTEM_HEALTH.lock().await.imu_is_up = is_up;
}
pub async fn update_baro_health(is_up: bool) {
    SYSTEM_HEALTH.lock().await.baro_is_up = is_up;
}
