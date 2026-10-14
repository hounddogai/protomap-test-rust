use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a ttl policy agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAgreement {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub quota_units: i64,
    pub cost: f64,
    pub reviewer: String,
    pub expires_at: i64,
    pub visible: bool,
    pub retries: i64,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub status: TtlPolicyAgreementStatus,
}

impl TtlPolicyAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyAgreementStatus) -> bool {
        match self.status {
            TtlPolicyAgreementStatus::Created => {
                match next {
                    TtlPolicyAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAgreementStatus::Validated => {
                return next == TtlPolicyAgreementStatus::Published || next == TtlPolicyAgreementStatus::Retired;
            }
            TtlPolicyAgreementStatus::Published => {
                return next == TtlPolicyAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy agreement {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "ttl policy agreement", self.code)
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

    pub fn weighted_exposure(&self) -> f64 {
        (self.cost * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyAgreement {
        TtlPolicyAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            quota_units: self.quota_units,
            cost: self.cost,
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            visible: self.visible,
            retries: self.retries,
            attributes: self.attributes.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy agreement in status created.
pub fn new_ttl_policy_agreement(code: &str, name: &str) -> TtlPolicyAgreement {
    TtlPolicyAgreement {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        quota_units: 0,
        cost: 0.0,
        reviewer: "".to_string(),
        expires_at: 0,
        visible: false,
        retries: 0,
        attributes: BTreeMap::new(),
        approved: false,
        status: TtlPolicyAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_ttl_policy_agreement("code-606", "name-247");
        assert_eq!(item.status, TtlPolicyAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_ttl_policy_agreement("code-606", "name-247");
        if item.move_to(TtlPolicyAgreementStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyAgreementStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_ttl_policy_agreement("code-606", "name-247");
        item.status = TtlPolicyAgreementStatus::Retired;
        assert!(
            item.move_to(TtlPolicyAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
