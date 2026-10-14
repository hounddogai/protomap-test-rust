use super::*;

use acme_kit::text;

/// Where a fallback binding milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBindingMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackBindingMilestoneStatus {
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

impl std::fmt::Display for FallbackBindingMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback binding milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestone {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub updated_at: i64,
    pub fingerprint: String,
    pub rate: f64,
    pub approved: bool,
    pub contact_email: String,
    pub segment: String,
    pub timezone: String,
    pub version: i64,
    pub status: FallbackBindingMilestoneStatus,
}

impl FallbackBindingMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBindingMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBindingMilestoneStatus) -> bool {
        match self.status {
            FallbackBindingMilestoneStatus::Open => {
                return next == FallbackBindingMilestoneStatus::InReview
                    || next == FallbackBindingMilestoneStatus::Resolved;
            }
            FallbackBindingMilestoneStatus::InReview => {
                return next == FallbackBindingMilestoneStatus::Reopened
                    || next == FallbackBindingMilestoneStatus::Archived;
            }
            FallbackBindingMilestoneStatus::Resolved => {
                match next {
                    FallbackBindingMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBindingMilestoneStatus::Reopened => {
                return next == FallbackBindingMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackBindingMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback binding milestone {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.version as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBindingMilestone {
        FallbackBindingMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            rate: self.rate,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            timezone: self.timezone.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback binding milestone in status open.
pub fn new_fallback_binding_milestone(code: &str, label: &str) -> FallbackBindingMilestone {
    FallbackBindingMilestone {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        updated_at: 0,
        fingerprint: "".to_string(),
        rate: 0.0,
        approved: false,
        contact_email: "".to_string(),
        segment: "".to_string(),
        timezone: "".to_string(),
        version: 0,
        status: FallbackBindingMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_binding_milestone("code-450", "label-439");
        assert_eq!(item.status, FallbackBindingMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_binding_milestone("code-450", "label-439");
        if item.move_to(FallbackBindingMilestoneStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBindingMilestoneStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_binding_milestone("code-450", "label-439");
        item.status = FallbackBindingMilestoneStatus::Archived;
        assert!(
            item.move_to(FallbackBindingMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
