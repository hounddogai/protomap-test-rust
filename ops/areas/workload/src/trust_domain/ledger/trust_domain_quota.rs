use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TrustDomainQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TrustDomainQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuota {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub channel: String,
    pub notes: String,
    pub burst: i64,
    pub status: TrustDomainQuotaStatus,
}

impl TrustDomainQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainQuotaStatus) -> bool {
        match self.status {
            TrustDomainQuotaStatus::Requested => {
                return next == TrustDomainQuotaStatus::Granted;
            }
            TrustDomainQuotaStatus::Granted => {
                match next {
                    TrustDomainQuotaStatus::Revoked | TrustDomainQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainQuotaStatus::Revoked => {
                match next {
                    TrustDomainQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: TrustDomainQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> TrustDomainQuota {
        TrustDomainQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            channel: self.channel.clone(),
            notes: self.notes.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a trust domain quota in status requested.
pub fn new_trust_domain_quota(code: &str, name: &str) -> TrustDomainQuota {
    TrustDomainQuota {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        channel: "".to_string(),
        notes: "".to_string(),
        burst: 0,
        status: TrustDomainQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_trust_domain_quota("code-362", "name-849");
        assert_eq!(item.status, TrustDomainQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_trust_domain_quota("code-362", "name-849");
        if item.move_to(TrustDomainQuotaStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainQuotaStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_trust_domain_quota("code-362", "name-849");
        item.status = TrustDomainQuotaStatus::Expired;
        assert!(
            item.move_to(TrustDomainQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
