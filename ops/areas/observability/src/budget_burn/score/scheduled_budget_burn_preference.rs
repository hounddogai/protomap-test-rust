use super::*;

use acme_kit::text;

/// Where a scheduled budget burn preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledBudgetBurnPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledBudgetBurnPreferenceStatus {
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

impl std::fmt::Display for ScheduledBudgetBurnPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled budget burn preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnPreference {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub priority: i64,
    pub updated_at: i64,
    pub threshold: f64,
    pub contact_email: String,
    pub currency: String,
    pub status: ScheduledBudgetBurnPreferenceStatus,
}

impl ScheduledBudgetBurnPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledBudgetBurnPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledBudgetBurnPreferenceStatus) -> bool {
        match self.status {
            ScheduledBudgetBurnPreferenceStatus::Open => {
                match next {
                    ScheduledBudgetBurnPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBudgetBurnPreferenceStatus::InReview => {
                return next == ScheduledBudgetBurnPreferenceStatus::Archived;
            }
            ScheduledBudgetBurnPreferenceStatus::Resolved => {
                match next {
                    ScheduledBudgetBurnPreferenceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBudgetBurnPreferenceStatus::Reopened => {
                return next == ScheduledBudgetBurnPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledBudgetBurnPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled budget burn preference {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled budget burn preference", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledBudgetBurnPreference {
        ScheduledBudgetBurnPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            updated_at: self.updated_at,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled budget burn preference in status open.
pub fn new_scheduled_budget_burn_preference(code: &str, label: &str) -> ScheduledBudgetBurnPreference {
    ScheduledBudgetBurnPreference {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        priority: 0,
        updated_at: 0,
        threshold: 0.0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        status: ScheduledBudgetBurnPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_budget_burn_preference("code-208", "label-92");
        assert_eq!(item.status, ScheduledBudgetBurnPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_scheduled_budget_burn_preference("code-208", "label-92");
        if item.move_to(ScheduledBudgetBurnPreferenceStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledBudgetBurnPreferenceStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_budget_burn_preference("code-208", "label-92");
        item.status = ScheduledBudgetBurnPreferenceStatus::Archived;
        assert!(
            item.move_to(ScheduledBudgetBurnPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
