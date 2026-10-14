use super::*;

use acme_kit::text;

/// Where a scheduled tenant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledTenantLedgerStatus {
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

impl std::fmt::Display for ScheduledTenantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled tenant ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantLedger {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub cluster: String,
    pub cost: f64,
    pub source: String,
    pub locale: String,
    pub status: ScheduledTenantLedgerStatus,
}

impl ScheduledTenantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantLedgerStatus) -> bool {
        match self.status {
            ScheduledTenantLedgerStatus::Draft => {
                match next {
                    ScheduledTenantLedgerStatus::Active | ScheduledTenantLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantLedgerStatus::Pending => {
                match next {
                    ScheduledTenantLedgerStatus::Active | ScheduledTenantLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantLedgerStatus::Active => {
                match next {
                    ScheduledTenantLedgerStatus::Suspended | ScheduledTenantLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantLedgerStatus::Suspended => {
                return next == ScheduledTenantLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTenantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.batch_size as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantLedger {
        ScheduledTenantLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            cluster: self.cluster.clone(),
            cost: self.cost,
            source: self.source.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled tenant ledger in status draft.
pub fn new_scheduled_tenant_ledger(code: &str, label: &str) -> ScheduledTenantLedger {
    ScheduledTenantLedger {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        cluster: "".to_string(),
        cost: 0.0,
        source: "".to_string(),
        locale: "".to_string(),
        status: ScheduledTenantLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_tenant_ledger("code-80", "label-75");
        assert_eq!(item.status, ScheduledTenantLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_scheduled_tenant_ledger("code-80", "label-75");
        if item.move_to(ScheduledTenantLedgerStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantLedgerStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_tenant_ledger("code-80", "label-75");
        item.status = ScheduledTenantLedgerStatus::Closed;
        assert!(
            item.move_to(ScheduledTenantLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
