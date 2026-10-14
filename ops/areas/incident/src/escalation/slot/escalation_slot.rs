use super::*;

use acme_kit::text;

/// Where a escalation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EscalationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EscalationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlot {
    pub code: String,
    pub label: String,
    pub title: String,
    pub starts_at: i64,
    pub description: String,
    pub status: EscalationSlotStatus,
}

impl EscalationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationSlotStatus) -> bool {
        match self.status {
            EscalationSlotStatus::Proposed => {
                match next {
                    EscalationSlotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationSlotStatus::Approved => {
                match next {
                    EscalationSlotStatus::Rejected | EscalationSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationSlotStatus::Rejected => {
                match next {
                    EscalationSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EscalationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> EscalationSlot {
        EscalationSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation slot in status proposed.
pub fn new_escalation_slot(code: &str, label: &str) -> EscalationSlot {
    EscalationSlot {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        starts_at: 0,
        description: "".to_string(),
        status: EscalationSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_escalation_slot("code-113", "label-607");
        assert_eq!(item.status, EscalationSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_escalation_slot("code-113", "label-607");
        if item.move_to(EscalationSlotStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationSlotStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_escalation_slot("code-113", "label-607");
        item.status = EscalationSlotStatus::Withdrawn;
        assert!(
            item.move_to(EscalationSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
