use super::*;

use acme_kit::text;

/// Where a pending suspension schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSuspensionScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingSuspensionScheduleStatus {
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

impl std::fmt::Display for PendingSuspensionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending suspension schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionSchedule {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub port: i64,
    pub category: String,
    pub label: String,
    pub status: PendingSuspensionScheduleStatus,
}

impl PendingSuspensionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSuspensionScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSuspensionScheduleStatus) -> bool {
        match self.status {
            PendingSuspensionScheduleStatus::Open => {
                return next == PendingSuspensionScheduleStatus::InReview;
            }
            PendingSuspensionScheduleStatus::InReview => {
                return next == PendingSuspensionScheduleStatus::Resolved
                    || next == PendingSuspensionScheduleStatus::Reopened;
            }
            PendingSuspensionScheduleStatus::Resolved => {
                match next {
                    PendingSuspensionScheduleStatus::Reopened | PendingSuspensionScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSuspensionScheduleStatus::Reopened => {
                return next == PendingSuspensionScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSuspensionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending suspension schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending suspension schedule", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSuspensionSchedule {
        PendingSuspensionSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            port: self.port,
            category: self.category.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending suspension schedule in status open.
pub fn new_pending_suspension_schedule(code: &str, name: &str) -> PendingSuspensionSchedule {
    PendingSuspensionSchedule {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        port: 0,
        category: "".to_string(),
        label: "".to_string(),
        status: PendingSuspensionScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_suspension_schedule("code-435", "name-32");
        assert_eq!(item.status, PendingSuspensionScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_suspension_schedule("code-435", "name-32");
        if item.move_to(PendingSuspensionScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSuspensionScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_suspension_schedule("code-435", "name-32");
        item.status = PendingSuspensionScheduleStatus::Archived;
        assert!(
            item.move_to(PendingSuspensionScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
