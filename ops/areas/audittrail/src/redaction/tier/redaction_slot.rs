use super::*;

use acme_kit::text;

/// Where a redaction slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionSlotStatus {
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

impl std::fmt::Display for RedactionSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlot {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub principal: String,
    pub description: String,
    pub burst: i64,
    pub category: String,
    pub owner_id: String,
    pub attempts: i64,
    pub status: RedactionSlotStatus,
}

impl RedactionSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionSlotStatus) -> bool {
        match self.status {
            RedactionSlotStatus::Planned => {
                return next == RedactionSlotStatus::Scheduled || next == RedactionSlotStatus::Completed;
            }
            RedactionSlotStatus::Scheduled => {
                return next == RedactionSlotStatus::InProgress;
            }
            RedactionSlotStatus::InProgress => {
                return next == RedactionSlotStatus::Completed;
            }
            RedactionSlotStatus::Completed => {
                return next == RedactionSlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.burst as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionSlot {
        RedactionSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            principal: self.principal.clone(),
            description: self.description.clone(),
            burst: self.burst,
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a redaction slot in status planned.
pub fn new_redaction_slot(code: &str, name: &str) -> RedactionSlot {
    RedactionSlot {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        principal: "".to_string(),
        description: "".to_string(),
        burst: 0,
        category: "".to_string(),
        owner_id: "".to_string(),
        attempts: 0,
        status: RedactionSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_slot("code-554", "name-816");
        assert_eq!(item.status, RedactionSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_redaction_slot("code-554", "name-816");
        if item.move_to(RedactionSlotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionSlotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_slot("code-554", "name-816");
        item.status = RedactionSlotStatus::Cancelled;
        assert!(
            item.move_to(RedactionSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
