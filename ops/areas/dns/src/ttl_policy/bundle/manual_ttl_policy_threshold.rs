use super::*;

use acme_kit::text;

/// Where a manual ttl policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTtlPolicyThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualTtlPolicyThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualTtlPolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual ttl policy threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThreshold {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub revision: i64,
    pub name: String,
    pub quantity: i64,
    pub retries: i64,
    pub locale: String,
    pub issuer: String,
    pub status: ManualTtlPolicyThresholdStatus,
}

impl ManualTtlPolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTtlPolicyThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTtlPolicyThresholdStatus) -> bool {
        match self.status {
            ManualTtlPolicyThresholdStatus::Requested => {
                return next == ManualTtlPolicyThresholdStatus::Revoked
                    || next == ManualTtlPolicyThresholdStatus::Expired;
            }
            ManualTtlPolicyThresholdStatus::Granted => {
                match next {
                    ManualTtlPolicyThresholdStatus::Revoked | ManualTtlPolicyThresholdStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyThresholdStatus::Revoked => {
                return next == ManualTtlPolicyThresholdStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualTtlPolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual ttl policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual ttl policy threshold", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTtlPolicyThreshold {
        ManualTtlPolicyThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            revision: self.revision,
            name: self.name.clone(),
            quantity: self.quantity,
            retries: self.retries,
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual ttl policy threshold in status requested.
pub fn new_manual_ttl_policy_threshold(code: &str, title: &str) -> ManualTtlPolicyThreshold {
    ManualTtlPolicyThreshold {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        revision: 0,
        name: "".to_string(),
        quantity: 0,
        retries: 0,
        locale: "".to_string(),
        issuer: "".to_string(),
        status: ManualTtlPolicyThresholdStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_ttl_policy_threshold("code-942", "title-24");
        assert_eq!(item.status, ManualTtlPolicyThresholdStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_ttl_policy_threshold("code-942", "title-24");
        if item.move_to(ManualTtlPolicyThresholdStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTtlPolicyThresholdStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_ttl_policy_threshold("code-942", "title-24");
        item.status = ManualTtlPolicyThresholdStatus::Expired;
        assert!(
            item.move_to(ManualTtlPolicyThresholdStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
