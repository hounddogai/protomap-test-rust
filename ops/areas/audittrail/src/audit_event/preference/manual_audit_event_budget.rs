use super::*;

use acme_kit::text;

/// Where a manual audit event budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAuditEventBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualAuditEventBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualAuditEventBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual audit event budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventBudget {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub region: String,
    pub expires_at: i64,
    pub cluster: String,
    pub discount: f64,
    pub status: ManualAuditEventBudgetStatus,
}

impl ManualAuditEventBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAuditEventBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAuditEventBudgetStatus) -> bool {
        match self.status {
            ManualAuditEventBudgetStatus::Requested => {
                return next == ManualAuditEventBudgetStatus::Granted || next == ManualAuditEventBudgetStatus::Revoked;
            }
            ManualAuditEventBudgetStatus::Granted => {
                return next == ManualAuditEventBudgetStatus::Revoked || next == ManualAuditEventBudgetStatus::Expired;
            }
            ManualAuditEventBudgetStatus::Revoked => {
                return next == ManualAuditEventBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAuditEventBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual audit event budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual audit event budget", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.discount * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualAuditEventBudget {
        ManualAuditEventBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            region: self.region.clone(),
            expires_at: self.expires_at,
            cluster: self.cluster.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual audit event budget in status requested.
pub fn new_manual_audit_event_budget(code: &str, label: &str) -> ManualAuditEventBudget {
    ManualAuditEventBudget {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        region: "".to_string(),
        expires_at: 0,
        cluster: "".to_string(),
        discount: 0.0,
        status: ManualAuditEventBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_audit_event_budget("code-420", "label-534");
        assert_eq!(item.status, ManualAuditEventBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_audit_event_budget("code-420", "label-534");
        if item.move_to(ManualAuditEventBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualAuditEventBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_audit_event_budget("code-420", "label-534");
        item.status = ManualAuditEventBudgetStatus::Expired;
        assert!(
            item.move_to(ManualAuditEventBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
