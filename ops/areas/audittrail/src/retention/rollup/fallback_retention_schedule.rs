use super::*;

use acme_kit::text;

/// Where a fallback retention schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRetentionScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackRetentionScheduleStatus {
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

impl std::fmt::Display for FallbackRetentionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback retention schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionSchedule {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub summary: String,
    pub updated_at: i64,
    pub quantity: i64,
    pub quota_units: i64,
    pub approved: bool,
    pub rate: f64,
    pub margin: f64,
    pub status: FallbackRetentionScheduleStatus,
}

impl FallbackRetentionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRetentionScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRetentionScheduleStatus) -> bool {
        match self.status {
            FallbackRetentionScheduleStatus::Open => {
                return next == FallbackRetentionScheduleStatus::InReview;
            }
            FallbackRetentionScheduleStatus::InReview => {
                match next {
                    FallbackRetentionScheduleStatus::Resolved | FallbackRetentionScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionScheduleStatus::Resolved => {
                return next == FallbackRetentionScheduleStatus::Reopened;
            }
            FallbackRetentionScheduleStatus::Reopened => {
                match next {
                    FallbackRetentionScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackRetentionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback retention schedule {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.batch_size as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRetentionSchedule {
        FallbackRetentionSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            summary: self.summary.clone(),
            updated_at: self.updated_at,
            quantity: self.quantity,
            quota_units: self.quota_units,
            approved: self.approved,
            rate: self.rate,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback retention schedule in status open.
pub fn new_fallback_retention_schedule(code: &str, title: &str) -> FallbackRetentionSchedule {
    FallbackRetentionSchedule {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        summary: "".to_string(),
        updated_at: 0,
        quantity: 0,
        quota_units: 0,
        approved: false,
        rate: 0.0,
        margin: 0.0,
        status: FallbackRetentionScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_retention_schedule("code-190", "title-983");
        assert_eq!(item.status, FallbackRetentionScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_retention_schedule("code-190", "title-983");
        if item.move_to(FallbackRetentionScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRetentionScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_retention_schedule("code-190", "title-983");
        item.status = FallbackRetentionScheduleStatus::Archived;
        assert!(
            item.move_to(FallbackRetentionScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
