use super::*;

use acme_kit::text;

/// Where a scheduled release budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReleaseBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledReleaseBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledReleaseBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled release budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudget {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub active: bool,
    pub checksum: String,
    pub maximum: f64,
    pub status: ScheduledReleaseBudgetStatus,
}

impl ScheduledReleaseBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReleaseBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReleaseBudgetStatus) -> bool {
        match self.status {
            ScheduledReleaseBudgetStatus::Proposed => {
                return next == ScheduledReleaseBudgetStatus::Approved
                    || next == ScheduledReleaseBudgetStatus::Rejected;
            }
            ScheduledReleaseBudgetStatus::Approved => {
                return next == ScheduledReleaseBudgetStatus::Rejected
                    || next == ScheduledReleaseBudgetStatus::Withdrawn;
            }
            ScheduledReleaseBudgetStatus::Rejected => {
                return next == ScheduledReleaseBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledReleaseBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled release budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled release budget", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.margin * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledReleaseBudget {
        ScheduledReleaseBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            active: self.active,
            checksum: self.checksum.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled release budget in status proposed.
pub fn new_scheduled_release_budget(code: &str, title: &str) -> ScheduledReleaseBudget {
    ScheduledReleaseBudget {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        external_ref: "".to_string(),
        ttl_seconds: 0,
        margin: 0.0,
        active: false,
        checksum: "".to_string(),
        maximum: 0.0,
        status: ScheduledReleaseBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_release_budget("code-120", "title-953");
        assert_eq!(item.status, ScheduledReleaseBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_release_budget("code-120", "title-953");
        if item.move_to(ScheduledReleaseBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledReleaseBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_release_budget("code-120", "title-953");
        item.status = ScheduledReleaseBudgetStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledReleaseBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
