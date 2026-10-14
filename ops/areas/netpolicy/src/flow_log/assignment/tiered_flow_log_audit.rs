use super::*;

use acme_kit::text;

/// Where a tiered flow log audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFlowLogAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredFlowLogAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredFlowLogAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered flow log audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAudit {
    pub code: String,
    pub title: String,
    pub description: String,
    pub summary: String,
    pub external_ref: String,
    pub weight: f64,
    pub budget: f64,
    pub locale: String,
    pub starts_at: i64,
    pub threshold: f64,
    pub name: String,
    pub status: TieredFlowLogAuditStatus,
}

impl TieredFlowLogAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFlowLogAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFlowLogAuditStatus) -> bool {
        match self.status {
            TieredFlowLogAuditStatus::Proposed => {
                match next {
                    TieredFlowLogAuditStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogAuditStatus::Approved => {
                match next {
                    TieredFlowLogAuditStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogAuditStatus::Rejected => {
                match next {
                    TieredFlowLogAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredFlowLogAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered flow log audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        (self.budget * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFlowLogAudit {
        TieredFlowLogAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            weight: self.weight,
            budget: self.budget,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            threshold: self.threshold,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered flow log audit in status proposed.
pub fn new_tiered_flow_log_audit(code: &str, title: &str) -> TieredFlowLogAudit {
    TieredFlowLogAudit {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        summary: "".to_string(),
        external_ref: "".to_string(),
        weight: 0.0,
        budget: 0.0,
        locale: "".to_string(),
        starts_at: 0,
        threshold: 0.0,
        name: "".to_string(),
        status: TieredFlowLogAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_flow_log_audit("code-931", "title-798");
        assert_eq!(item.status, TieredFlowLogAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_flow_log_audit("code-931", "title-798");
        if item.move_to(TieredFlowLogAuditStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFlowLogAuditStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_flow_log_audit("code-931", "title-798");
        item.status = TieredFlowLogAuditStatus::Withdrawn;
        assert!(
            item.move_to(TieredFlowLogAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
