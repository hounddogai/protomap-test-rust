use super::*;

use acme_kit::text;

/// Where a pending plan audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPlanAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingPlanAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingPlanAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending plan audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAudit {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub scope: String,
    pub tenant_id: String,
    pub active: bool,
    pub channel: String,
    pub external_ref: String,
    pub rate: f64,
    pub checksum: String,
    pub status: PendingPlanAuditStatus,
}

impl PendingPlanAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPlanAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPlanAuditStatus) -> bool {
        match self.status {
            PendingPlanAuditStatus::Proposed => {
                return next == PendingPlanAuditStatus::Approved;
            }
            PendingPlanAuditStatus::Approved => {
                match next {
                    PendingPlanAuditStatus::Rejected | PendingPlanAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPlanAuditStatus::Rejected => {
                return next == PendingPlanAuditStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPlanAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending plan audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending plan audit", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.rate * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPlanAudit {
        PendingPlanAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            scope: self.scope.clone(),
            tenant_id: self.tenant_id.clone(),
            active: self.active,
            channel: self.channel.clone(),
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending plan audit in status proposed.
pub fn new_pending_plan_audit(code: &str, title: &str) -> PendingPlanAudit {
    PendingPlanAudit {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        scope: "".to_string(),
        tenant_id: "".to_string(),
        active: false,
        channel: "".to_string(),
        external_ref: "".to_string(),
        rate: 0.0,
        checksum: "".to_string(),
        status: PendingPlanAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_plan_audit("code-403", "title-46");
        assert_eq!(item.status, PendingPlanAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_plan_audit("code-403", "title-46");
        if item.move_to(PendingPlanAuditStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPlanAuditStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_plan_audit("code-403", "title-46");
        item.status = PendingPlanAuditStatus::Withdrawn;
        assert!(
            item.move_to(PendingPlanAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
