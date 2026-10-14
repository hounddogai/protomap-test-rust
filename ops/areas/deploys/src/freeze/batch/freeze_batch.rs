use super::*;

use acme_kit::text;

/// Where a freeze batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FreezeBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FreezeBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatch {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub retries: i64,
    pub timezone: String,
    pub status: FreezeBatchStatus,
}

impl FreezeBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeBatchStatus) -> bool {
        match self.status {
            FreezeBatchStatus::Planned => {
                return next == FreezeBatchStatus::Scheduled;
            }
            FreezeBatchStatus::Scheduled => {
                return next == FreezeBatchStatus::InProgress;
            }
            FreezeBatchStatus::InProgress => {
                return next == FreezeBatchStatus::Cancelled;
            }
            FreezeBatchStatus::Completed => {
                match next {
                    FreezeBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FreezeBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> FreezeBatch {
        FreezeBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            retries: self.retries,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze batch in status planned.
pub fn new_freeze_batch(code: &str, name: &str) -> FreezeBatch {
    FreezeBatch {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        retries: 0,
        timezone: "".to_string(),
        status: FreezeBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_freeze_batch("code-825", "name-792");
        assert_eq!(item.status, FreezeBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_freeze_batch("code-825", "name-792");
        if item.move_to(FreezeBatchStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeBatchStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_freeze_batch("code-825", "name-792");
        item.status = FreezeBatchStatus::Cancelled;
        assert!(
            item.move_to(FreezeBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
