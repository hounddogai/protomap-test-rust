use super::*;

use acme_kit::text;

/// Where a freeze milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeMilestoneStatus {
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

impl std::fmt::Display for FreezeMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMilestone {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub reviewer: String,
    pub phone: String,
    pub country: String,
    pub amount: f64,
    pub region: String,
    pub margin: f64,
    pub locale: String,
    pub contact_email: String,
    pub status: FreezeMilestoneStatus,
}

impl FreezeMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeMilestoneStatus) -> bool {
        match self.status {
            FreezeMilestoneStatus::New => {
                match next {
                    FreezeMilestoneStatus::Queued | FreezeMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeMilestoneStatus::Queued => {
                return next == FreezeMilestoneStatus::Running || next == FreezeMilestoneStatus::Failed;
            }
            FreezeMilestoneStatus::Running => {
                return next == FreezeMilestoneStatus::Failed;
            }
            FreezeMilestoneStatus::Done => {
                return next == FreezeMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.amount * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeMilestone {
        FreezeMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            reviewer: self.reviewer.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            amount: self.amount,
            region: self.region.clone(),
            margin: self.margin,
            locale: self.locale.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a freeze milestone in status new.
pub fn new_freeze_milestone(code: &str, label: &str) -> FreezeMilestone {
    FreezeMilestone {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        reviewer: "".to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        amount: 0.0,
        region: "".to_string(),
        margin: 0.0,
        locale: "".to_string(),
        contact_email: "".to_string(),
        status: FreezeMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_milestone("code-762", "label-138");
        assert_eq!(item.status, FreezeMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_freeze_milestone("code-762", "label-138");
        if item.move_to(FreezeMilestoneStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeMilestoneStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_milestone("code-762", "label-138");
        item.status = FreezeMilestoneStatus::Failed;
        assert!(
            item.move_to(FreezeMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
