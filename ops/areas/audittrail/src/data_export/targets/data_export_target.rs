use super::*;

use acme_kit::text;

/// Where a data export target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportTargetStatus {
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

impl std::fmt::Display for DataExportTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a data export target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTarget {
    pub code: String,
    pub name: String,
    pub title: String,
    pub burst: i64,
    pub ratio: f64,
    pub updated_at: i64,
    pub margin: f64,
    pub cost: f64,
    pub budget: f64,
    pub grace_days: i64,
    pub status: DataExportTargetStatus,
}

impl DataExportTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportTargetStatus) -> bool {
        match self.status {
            DataExportTargetStatus::Idle => {
                match next {
                    DataExportTargetStatus::Ready | DataExportTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportTargetStatus::Warming => {
                match next {
                    DataExportTargetStatus::Draining | DataExportTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportTargetStatus::Ready => {
                return next == DataExportTargetStatus::Draining || next == DataExportTargetStatus::Stopped;
            }
            DataExportTargetStatus::Draining => {
                return next == DataExportTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export target", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportTarget {
        DataExportTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            burst: self.burst,
            ratio: self.ratio,
            updated_at: self.updated_at,
            margin: self.margin,
            cost: self.cost,
            budget: self.budget,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export target in status idle.
pub fn new_data_export_target(code: &str, name: &str) -> DataExportTarget {
    DataExportTarget {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        burst: 0,
        ratio: 0.0,
        updated_at: 0,
        margin: 0.0,
        cost: 0.0,
        budget: 0.0,
        grace_days: 0,
        status: DataExportTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_data_export_target("code-937", "name-221");
        assert_eq!(item.status, DataExportTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_data_export_target("code-937", "name-221");
        if item.move_to(DataExportTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_data_export_target("code-937", "name-221");
        item.status = DataExportTargetStatus::Stopped;
        assert!(
            item.move_to(DataExportTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
