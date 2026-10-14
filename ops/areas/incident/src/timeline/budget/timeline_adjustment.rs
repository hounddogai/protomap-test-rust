use super::*;

use acme_kit::text;

/// Where a timeline adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline adjustment as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustment {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub updated_at: i64,
    pub visible: bool,
    pub quota_units: i64,
    pub ratio: f64,
    pub owner_id: String,
    pub batch_size: i64,
    pub status: TimelineAdjustmentStatus,
}

impl TimelineAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineAdjustmentStatus) -> bool {
        match self.status {
            TimelineAdjustmentStatus::Proposed => {
                match next {
                    TimelineAdjustmentStatus::Approved | TimelineAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAdjustmentStatus::Approved => {
                return next == TimelineAdjustmentStatus::Rejected || next == TimelineAdjustmentStatus::Withdrawn;
            }
            TimelineAdjustmentStatus::Rejected => {
                match next {
                    TimelineAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TimelineAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline adjustment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineAdjustment {
        TimelineAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            updated_at: self.updated_at,
            visible: self.visible,
            quota_units: self.quota_units,
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline adjustment in status proposed.
pub fn new_timeline_adjustment(code: &str, name: &str) -> TimelineAdjustment {
    TimelineAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        updated_at: 0,
        visible: false,
        quota_units: 0,
        ratio: 0.0,
        owner_id: "".to_string(),
        batch_size: 0,
        status: TimelineAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_timeline_adjustment("code-83", "name-714");
        assert_eq!(item.status, TimelineAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_timeline_adjustment("code-83", "name-714");
        if item.move_to(TimelineAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_timeline_adjustment("code-83", "name-714");
        item.status = TimelineAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(TimelineAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
