use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted trust domain checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTrustDomainCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedTrustDomainCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedTrustDomainCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted trust domain checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainCheckpoint {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub attempts: i64,
    pub country: String,
    pub maximum: f64,
    pub budget: f64,
    pub priority: i64,
    pub fingerprint: String,
    pub status: WeightedTrustDomainCheckpointStatus,
}

impl WeightedTrustDomainCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTrustDomainCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTrustDomainCheckpointStatus) -> bool {
        match self.status {
            WeightedTrustDomainCheckpointStatus::Created => {
                return next == WeightedTrustDomainCheckpointStatus::Validated
                    || next == WeightedTrustDomainCheckpointStatus::Published;
            }
            WeightedTrustDomainCheckpointStatus::Validated => {
                return next == WeightedTrustDomainCheckpointStatus::Published
                    || next == WeightedTrustDomainCheckpointStatus::Retired;
            }
            WeightedTrustDomainCheckpointStatus::Published => {
                return next == WeightedTrustDomainCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTrustDomainCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted trust domain checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted trust domain checkpoint", self.code)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.attempts as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTrustDomainCheckpoint {
        WeightedTrustDomainCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            attempts: self.attempts,
            country: self.country.clone(),
            maximum: self.maximum,
            budget: self.budget,
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted trust domain checkpoint in status created.
pub fn new_weighted_trust_domain_checkpoint(code: &str, title: &str) -> WeightedTrustDomainCheckpoint {
    WeightedTrustDomainCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        attempts: 0,
        country: "".to_string(),
        maximum: 0.0,
        budget: 0.0,
        priority: 0,
        fingerprint: "".to_string(),
        status: WeightedTrustDomainCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_trust_domain_checkpoint("code-374", "title-25");
        assert_eq!(item.status, WeightedTrustDomainCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_trust_domain_checkpoint("code-374", "title-25");
        if item.move_to(WeightedTrustDomainCheckpointStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTrustDomainCheckpointStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_trust_domain_checkpoint("code-374", "title-25");
        item.status = WeightedTrustDomainCheckpointStatus::Retired;
        assert!(
            item.move_to(WeightedTrustDomainCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
