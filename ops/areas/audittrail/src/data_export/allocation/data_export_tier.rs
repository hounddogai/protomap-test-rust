use super::*;

use acme_kit::text;

/// Where a data export tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportTierStatus {
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

impl std::fmt::Display for DataExportTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTier {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub channel: String,
    pub rotation_days: i64,
    pub minimum: f64,
    pub batch_size: i64,
    pub status: DataExportTierStatus,
}

impl DataExportTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportTierStatus) -> bool {
        match self.status {
            DataExportTierStatus::Idle => {
                return next == DataExportTierStatus::Ready || next == DataExportTierStatus::Draining;
            }
            DataExportTierStatus::Warming => {
                match next {
                    DataExportTierStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportTierStatus::Ready => {
                return next == DataExportTierStatus::Draining || next == DataExportTierStatus::Stopped;
            }
            DataExportTierStatus::Draining => {
                return next == DataExportTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportTier {
        DataExportTier {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            minimum: self.minimum,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a data export tier in status idle.
pub fn new_data_export_tier(code: &str, label: &str) -> DataExportTier {
    DataExportTier {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        channel: "".to_string(),
        rotation_days: 0,
        minimum: 0.0,
        batch_size: 0,
        status: DataExportTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_data_export_tier("code-617", "label-659");
        assert_eq!(item.status, DataExportTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_data_export_tier("code-617", "label-659");
        if item.move_to(DataExportTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_data_export_tier("code-617", "label-659");
        item.status = DataExportTierStatus::Stopped;
        assert!(
            item.move_to(DataExportTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
