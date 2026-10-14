use super::*;

use acme_kit::text;

/// Where a tenant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TenantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedger {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub rotation_days: i64,
    pub ratio: f64,
    pub cluster: String,
    pub owner_id: String,
    pub scope: String,
    pub expires_at: i64,
    pub status: TenantLedgerStatus,
}

impl TenantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantLedgerStatus) -> bool {
        match self.status {
            TenantLedgerStatus::Open => {
                return next == TenantLedgerStatus::Resolved || next == TenantLedgerStatus::Archived;
            }
            TenantLedgerStatus::InReview => {
                match next {
                    TenantLedgerStatus::Resolved | TenantLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantLedgerStatus::Resolved => {
                match next {
                    TenantLedgerStatus::Reopened | TenantLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantLedgerStatus::Reopened => {
                match next {
                    TenantLedgerStatus::Archived => {
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

    pub fn move_to(&mut self, next: TenantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant ledger", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantLedger {
        TenantLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            cluster: self.cluster.clone(),
            owner_id: self.owner_id.clone(),
            scope: self.scope.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tenant ledger in status open.
pub fn new_tenant_ledger(code: &str, title: &str) -> TenantLedger {
    TenantLedger {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        rotation_days: 0,
        ratio: 0.0,
        cluster: "".to_string(),
        owner_id: "".to_string(),
        scope: "".to_string(),
        expires_at: 0,
        status: TenantLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_ledger("code-319", "title-190");
        assert_eq!(item.status, TenantLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_tenant_ledger("code-319", "title-190");
        if item.move_to(TenantLedgerStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantLedgerStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_ledger("code-319", "title-190");
        item.status = TenantLedgerStatus::Archived;
        assert!(item.move_to(TenantLedgerStatus::Open).is_err(), "expected move to fail");
    }
}
