use super::*;

use acme_kit::text;

/// Where a slo adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SloAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SloAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAdjustment {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub margin: f64,
    pub summary: String,
    pub key_id: String,
    pub floor_value: f64,
    pub approved: bool,
    pub grace_days: i64,
    pub cluster: String,
    pub quota_units: i64,
    pub status: SloAdjustmentStatus,
}

impl SloAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloAdjustmentStatus) -> bool {
        match self.status {
            SloAdjustmentStatus::New => {
                match next {
                    SloAdjustmentStatus::Queued | SloAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloAdjustmentStatus::Queued => {
                return next == SloAdjustmentStatus::Running || next == SloAdjustmentStatus::Done;
            }
            SloAdjustmentStatus::Running => {
                return next == SloAdjustmentStatus::Done || next == SloAdjustmentStatus::Failed;
            }
            SloAdjustmentStatus::Done => {
                return next == SloAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloAdjustment {
        SloAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            margin: self.margin,
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            floor_value: self.floor_value,
            approved: self.approved,
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a slo adjustment in status new.
pub fn new_slo_adjustment(code: &str, title: &str) -> SloAdjustment {
    SloAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        margin: 0.0,
        summary: "".to_string(),
        key_id: "".to_string(),
        floor_value: 0.0,
        approved: false,
        grace_days: 0,
        cluster: "".to_string(),
        quota_units: 0,
        status: SloAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_slo_adjustment("code-942", "title-710");
        assert_eq!(item.status, SloAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_slo_adjustment("code-942", "title-710");
        if item.move_to(SloAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_slo_adjustment("code-942", "title-710");
        item.status = SloAdjustmentStatus::Failed;
        assert!(item.move_to(SloAdjustmentStatus::New).is_err(), "expected move to fail");
    }
}
