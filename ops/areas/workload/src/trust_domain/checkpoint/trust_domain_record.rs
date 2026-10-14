use super::*;

use acme_kit::text;

/// Where a trust domain record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TrustDomainRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TrustDomainRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain record as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecord {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub active: bool,
    pub quota_units: i64,
    pub currency: String,
    pub namespace: String,
    pub status: TrustDomainRecordStatus,
}

impl TrustDomainRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainRecordStatus) -> bool {
        match self.status {
            TrustDomainRecordStatus::Requested => {
                return next == TrustDomainRecordStatus::Revoked || next == TrustDomainRecordStatus::Expired;
            }
            TrustDomainRecordStatus::Granted => {
                return next == TrustDomainRecordStatus::Revoked;
            }
            TrustDomainRecordStatus::Revoked => {
                return next == TrustDomainRecordStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quota_units as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainRecord {
        TrustDomainRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            active: self.active,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a trust domain record in status requested.
pub fn new_trust_domain_record(code: &str, label: &str) -> TrustDomainRecord {
    TrustDomainRecord {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        active: false,
        quota_units: 0,
        currency: "".to_string(),
        namespace: "".to_string(),
        status: TrustDomainRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_trust_domain_record("code-487", "label-74");
        assert_eq!(item.status, TrustDomainRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_trust_domain_record("code-487", "label-74");
        if item.move_to(TrustDomainRecordStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainRecordStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_trust_domain_record("code-487", "label-74");
        item.status = TrustDomainRecordStatus::Expired;
        assert!(
            item.move_to(TrustDomainRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
