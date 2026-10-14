use super::*;

use acme_kit::text;

/// Where a tiered data export rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDataExportRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredDataExportRollupStatus {
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

impl std::fmt::Display for TieredDataExportRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered data export rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollup {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub capacity: i64,
    pub notes: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub active: bool,
    pub summary: String,
    pub margin: f64,
    pub quota_units: i64,
    pub status: TieredDataExportRollupStatus,
}

impl TieredDataExportRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDataExportRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDataExportRollupStatus) -> bool {
        match self.status {
            TieredDataExportRollupStatus::Idle => {
                match next {
                    TieredDataExportRollupStatus::Draining | TieredDataExportRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDataExportRollupStatus::Warming => {
                match next {
                    TieredDataExportRollupStatus::Draining | TieredDataExportRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDataExportRollupStatus::Ready => {
                return next == TieredDataExportRollupStatus::Stopped;
            }
            TieredDataExportRollupStatus::Draining => {
                return next == TieredDataExportRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredDataExportRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered data export rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDataExportRollup {
        TieredDataExportRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            capacity: self.capacity,
            notes: self.notes.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            active: self.active,
            summary: self.summary.clone(),
            margin: self.margin,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered data export rollup in status idle.
pub fn new_tiered_data_export_rollup(code: &str, title: &str) -> TieredDataExportRollup {
    TieredDataExportRollup {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        capacity: 0,
        notes: "".to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        active: false,
        summary: "".to_string(),
        margin: 0.0,
        quota_units: 0,
        status: TieredDataExportRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_data_export_rollup("code-959", "title-899");
        assert_eq!(item.status, TieredDataExportRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tiered_data_export_rollup("code-959", "title-899");
        if item.move_to(TieredDataExportRollupStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDataExportRollupStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_data_export_rollup("code-959", "title-899");
        item.status = TieredDataExportRollupStatus::Stopped;
        assert!(
            item.move_to(TieredDataExportRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
