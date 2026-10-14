use super::*;

use acme_kit::text;

/// Where a lenient slo schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientSloScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientSloScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient slo schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloSchedule {
    pub code: String,
    pub title: String,
    pub host: String,
    pub currency: String,
    pub unit: String,
    pub key_id: String,
    pub grace_days: i64,
    pub sequence: i64,
    pub status: LenientSloScheduleStatus,
}

impl LenientSloSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloScheduleStatus) -> bool {
        match self.status {
            LenientSloScheduleStatus::Open => {
                return next == LenientSloScheduleStatus::Resolved || next == LenientSloScheduleStatus::Archived;
            }
            LenientSloScheduleStatus::InReview => {
                return next == LenientSloScheduleStatus::Resolved || next == LenientSloScheduleStatus::Reopened;
            }
            LenientSloScheduleStatus::Resolved => {
                return next == LenientSloScheduleStatus::Reopened;
            }
            LenientSloScheduleStatus::Reopened => {
                return next == LenientSloScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSloScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient slo schedule", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSloSchedule {
        LenientSloSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            currency: self.currency.clone(),
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient slo schedule in status open.
pub fn new_lenient_slo_schedule(code: &str, title: &str) -> LenientSloSchedule {
    LenientSloSchedule {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        currency: "".to_string(),
        unit: "".to_string(),
        key_id: "".to_string(),
        grace_days: 0,
        sequence: 0,
        status: LenientSloScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_slo_schedule("code-134", "title-617");
        assert_eq!(item.status, LenientSloScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_slo_schedule("code-134", "title-617");
        if item.move_to(LenientSloScheduleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloScheduleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_slo_schedule("code-134", "title-617");
        item.status = LenientSloScheduleStatus::Archived;
        assert!(
            item.move_to(LenientSloScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
