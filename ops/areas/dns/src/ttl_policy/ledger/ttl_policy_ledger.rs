use super::*;

use acme_kit::text;

/// Where a ttl policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TtlPolicyLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TtlPolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyLedger {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub cluster: String,
    pub starts_at: i64,
    pub created_at: i64,
    pub contact_email: String,
    pub title: String,
    pub status: TtlPolicyLedgerStatus,
}

impl TtlPolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyLedgerStatus) -> bool {
        match self.status {
            TtlPolicyLedgerStatus::Requested => {
                match next {
                    TtlPolicyLedgerStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyLedgerStatus::Granted => {
                match next {
                    TtlPolicyLedgerStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyLedgerStatus::Revoked => {
                return next == TtlPolicyLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy ledger {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.starts_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyLedger {
        TtlPolicyLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            created_at: self.created_at,
            contact_email: self.contact_email.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a ttl policy ledger in status requested.
pub fn new_ttl_policy_ledger(code: &str, label: &str) -> TtlPolicyLedger {
    TtlPolicyLedger {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        cluster: "".to_string(),
        starts_at: 0,
        created_at: 0,
        contact_email: "".to_string(),
        title: "".to_string(),
        status: TtlPolicyLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_ttl_policy_ledger("code-297", "label-993");
        assert_eq!(item.status, TtlPolicyLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_ttl_policy_ledger("code-297", "label-993");
        if item.move_to(TtlPolicyLedgerStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyLedgerStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_ttl_policy_ledger("code-297", "label-993");
        item.status = TtlPolicyLedgerStatus::Expired;
        assert!(
            item.move_to(TtlPolicyLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
