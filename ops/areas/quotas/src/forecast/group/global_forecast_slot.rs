use super::*;

use acme_kit::text;

/// Where a global forecast slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalForecastSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalForecastSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for GlobalForecastSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global forecast slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastSlot {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub host: String,
    pub updated_at: i64,
    pub margin: f64,
    pub batch_size: i64,
    pub port: i64,
    pub checksum: String,
    pub version: i64,
    pub status: GlobalForecastSlotStatus,
}

impl GlobalForecastSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalForecastSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalForecastSlotStatus) -> bool {
        match self.status {
            GlobalForecastSlotStatus::Idle => {
                match next {
                    GlobalForecastSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastSlotStatus::Warming => {
                match next {
                    GlobalForecastSlotStatus::Draining | GlobalForecastSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastSlotStatus::Ready => {
                match next {
                    GlobalForecastSlotStatus::Draining | GlobalForecastSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastSlotStatus::Draining => {
                match next {
                    GlobalForecastSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalForecastSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global forecast slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quota_units as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalForecastSlot {
        GlobalForecastSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            host: self.host.clone(),
            updated_at: self.updated_at,
            margin: self.margin,
            batch_size: self.batch_size,
            port: self.port,
            checksum: self.checksum.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global forecast slot in status idle.
pub fn new_global_forecast_slot(code: &str, label: &str) -> GlobalForecastSlot {
    GlobalForecastSlot {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        host: "".to_string(),
        updated_at: 0,
        margin: 0.0,
        batch_size: 0,
        port: 0,
        checksum: "".to_string(),
        version: 0,
        status: GlobalForecastSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_forecast_slot("code-270", "label-473");
        assert_eq!(item.status, GlobalForecastSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_global_forecast_slot("code-270", "label-473");
        if item.move_to(GlobalForecastSlotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalForecastSlotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_forecast_slot("code-270", "label-473");
        item.status = GlobalForecastSlotStatus::Stopped;
        assert!(
            item.move_to(GlobalForecastSlotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
