use super::*;

use acme_kit::text;

/// Where a global trust domain quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTrustDomainQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalTrustDomainQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalTrustDomainQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global trust domain quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainQuota {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub pinned: bool,
    pub quota_units: i64,
    pub segment: String,
    pub status: GlobalTrustDomainQuotaStatus,
}

impl GlobalTrustDomainQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTrustDomainQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTrustDomainQuotaStatus) -> bool {
        match self.status {
            GlobalTrustDomainQuotaStatus::Requested => {
                match next {
                    GlobalTrustDomainQuotaStatus::Revoked | GlobalTrustDomainQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTrustDomainQuotaStatus::Granted => {
                match next {
                    GlobalTrustDomainQuotaStatus::Revoked | GlobalTrustDomainQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTrustDomainQuotaStatus::Revoked => {
                return next == GlobalTrustDomainQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTrustDomainQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global trust domain quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global trust domain quota", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.quota_units as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTrustDomainQuota {
        GlobalTrustDomainQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            pinned: self.pinned,
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global trust domain quota in status requested.
pub fn new_global_trust_domain_quota(code: &str, name: &str) -> GlobalTrustDomainQuota {
    GlobalTrustDomainQuota {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        pinned: false,
        quota_units: 0,
        segment: "".to_string(),
        status: GlobalTrustDomainQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_trust_domain_quota("code-799", "name-880");
        assert_eq!(item.status, GlobalTrustDomainQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_global_trust_domain_quota("code-799", "name-880");
        if item.move_to(GlobalTrustDomainQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTrustDomainQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_trust_domain_quota("code-799", "name-880");
        item.status = GlobalTrustDomainQuotaStatus::Expired;
        assert!(
            item.move_to(GlobalTrustDomainQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
