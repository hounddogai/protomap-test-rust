use super::*;

use acme_kit::text;

/// Where a redaction allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionAllocationStatus {
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

impl std::fmt::Display for RedactionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAllocation {
    pub code: String,
    pub label: String,
    pub source: String,
    pub notes: String,
    pub summary: String,
    pub quantity: i64,
    pub timezone: String,
    pub discount: f64,
    pub status: RedactionAllocationStatus,
}

impl RedactionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionAllocationStatus) -> bool {
        match self.status {
            RedactionAllocationStatus::New => {
                match next {
                    RedactionAllocationStatus::Queued | RedactionAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAllocationStatus::Queued => {
                match next {
                    RedactionAllocationStatus::Done | RedactionAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAllocationStatus::Running => {
                return next == RedactionAllocationStatus::Done || next == RedactionAllocationStatus::Failed;
            }
            RedactionAllocationStatus::Done => {
                return next == RedactionAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "redaction allocation", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quantity as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionAllocation {
        RedactionAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            notes: self.notes.clone(),
            summary: self.summary.clone(),
            quantity: self.quantity,
            timezone: self.timezone.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a redaction allocation in status new.
pub fn new_redaction_allocation(code: &str, label: &str) -> RedactionAllocation {
    RedactionAllocation {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        notes: "".to_string(),
        summary: "".to_string(),
        quantity: 0,
        timezone: "".to_string(),
        discount: 0.0,
        status: RedactionAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_redaction_allocation("code-758", "label-161");
        assert_eq!(item.status, RedactionAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_redaction_allocation("code-758", "label-161");
        if item.move_to(RedactionAllocationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionAllocationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_redaction_allocation("code-758", "label-161");
        item.status = RedactionAllocationStatus::Failed;
        assert!(
            item.move_to(RedactionAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
