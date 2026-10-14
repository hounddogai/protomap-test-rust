use super::*;

use acme_kit::text;

/// Where a tiered peering audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPeeringAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredPeeringAuditStatus {
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

impl std::fmt::Display for TieredPeeringAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered peering audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringAudit {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub external_ref: String,
    pub ratio: f64,
    pub key_id: String,
    pub status: TieredPeeringAuditStatus,
}

impl TieredPeeringAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPeeringAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPeeringAuditStatus) -> bool {
        match self.status {
            TieredPeeringAuditStatus::Planned => {
                return next == TieredPeeringAuditStatus::Scheduled || next == TieredPeeringAuditStatus::Cancelled;
            }
            TieredPeeringAuditStatus::Scheduled => {
                return next == TieredPeeringAuditStatus::Completed;
            }
            TieredPeeringAuditStatus::InProgress => {
                return next == TieredPeeringAuditStatus::Completed || next == TieredPeeringAuditStatus::Cancelled;
            }
            TieredPeeringAuditStatus::Completed => {
                return next == TieredPeeringAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPeeringAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered peering audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.revision as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPeeringAudit {
        TieredPeeringAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered peering audit in status planned.
pub fn new_tiered_peering_audit(code: &str, name: &str) -> TieredPeeringAudit {
    TieredPeeringAudit {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        external_ref: "".to_string(),
        ratio: 0.0,
        key_id: "".to_string(),
        status: TieredPeeringAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_peering_audit("code-688", "name-230");
        assert_eq!(item.status, TieredPeeringAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tiered_peering_audit("code-688", "name-230");
        if item.move_to(TieredPeeringAuditStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPeeringAuditStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_peering_audit("code-688", "name-230");
        item.status = TieredPeeringAuditStatus::Cancelled;
        assert!(
            item.move_to(TieredPeeringAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
