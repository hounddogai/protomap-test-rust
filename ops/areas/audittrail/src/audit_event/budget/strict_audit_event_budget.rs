use super::*;

use acme_kit::text;

/// Where a strict audit event budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAuditEventBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictAuditEventBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictAuditEventBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict audit event budget as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventBudget {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub locale: String,
    pub revision: i64,
    pub created_at: i64,
    pub amount: f64,
    pub status: StrictAuditEventBudgetStatus,
}

impl StrictAuditEventBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAuditEventBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAuditEventBudgetStatus) -> bool {
        match self.status {
            StrictAuditEventBudgetStatus::Draft => {
                match next {
                    StrictAuditEventBudgetStatus::Active | StrictAuditEventBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAuditEventBudgetStatus::Pending => {
                return next == StrictAuditEventBudgetStatus::Closed;
            }
            StrictAuditEventBudgetStatus::Active => {
                match next {
                    StrictAuditEventBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAuditEventBudgetStatus::Suspended => {
                match next {
                    StrictAuditEventBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictAuditEventBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict audit event budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAuditEventBudget {
        StrictAuditEventBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            locale: self.locale.clone(),
            revision: self.revision,
            created_at: self.created_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict audit event budget in status draft.
pub fn new_strict_audit_event_budget(code: &str, name: &str) -> StrictAuditEventBudget {
    StrictAuditEventBudget {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        locale: "".to_string(),
        revision: 0,
        created_at: 0,
        amount: 0.0,
        status: StrictAuditEventBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_audit_event_budget("code-657", "name-281");
        assert_eq!(item.status, StrictAuditEventBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_audit_event_budget("code-657", "name-281");
        if item.move_to(StrictAuditEventBudgetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAuditEventBudgetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_audit_event_budget("code-657", "name-281");
        item.status = StrictAuditEventBudgetStatus::Closed;
        assert!(
            item.move_to(StrictAuditEventBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
