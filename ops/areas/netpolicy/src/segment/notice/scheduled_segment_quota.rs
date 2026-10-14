use super::*;

use acme_kit::text;

/// Where a scheduled segment quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSegmentQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledSegmentQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledSegmentQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled segment quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentQuota {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub window_days: i64,
    pub margin: f64,
    pub capacity: i64,
    pub batch_size: i64,
    pub status: ScheduledSegmentQuotaStatus,
}

impl ScheduledSegmentQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSegmentQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSegmentQuotaStatus) -> bool {
        match self.status {
            ScheduledSegmentQuotaStatus::Proposed => {
                return next == ScheduledSegmentQuotaStatus::Approved || next == ScheduledSegmentQuotaStatus::Rejected;
            }
            ScheduledSegmentQuotaStatus::Approved => {
                return next == ScheduledSegmentQuotaStatus::Withdrawn;
            }
            ScheduledSegmentQuotaStatus::Rejected => {
                match next {
                    ScheduledSegmentQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledSegmentQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled segment quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.batch_size as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSegmentQuota {
        ScheduledSegmentQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            window_days: self.window_days,
            margin: self.margin,
            capacity: self.capacity,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled segment quota in status proposed.
pub fn new_scheduled_segment_quota(code: &str, label: &str) -> ScheduledSegmentQuota {
    ScheduledSegmentQuota {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        window_days: 0,
        margin: 0.0,
        capacity: 0,
        batch_size: 0,
        status: ScheduledSegmentQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_segment_quota("code-392", "label-195");
        assert_eq!(item.status, ScheduledSegmentQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_segment_quota("code-392", "label-195");
        if item.move_to(ScheduledSegmentQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSegmentQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_segment_quota("code-392", "label-195");
        item.status = ScheduledSegmentQuotaStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledSegmentQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
