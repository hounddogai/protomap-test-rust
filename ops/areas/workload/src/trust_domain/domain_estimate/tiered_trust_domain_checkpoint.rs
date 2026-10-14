use super::*;

use acme_kit::text;

/// Where a tiered trust domain checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTrustDomainCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredTrustDomainCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredTrustDomainCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered trust domain checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpoint {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub audience: String,
    pub minimum: f64,
    pub starts_at: i64,
    pub updated_at: i64,
    pub contact_email: String,
    pub budget: f64,
    pub retries: i64,
    pub ratio: f64,
    pub status: TieredTrustDomainCheckpointStatus,
}

impl TieredTrustDomainCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTrustDomainCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTrustDomainCheckpointStatus) -> bool {
        match self.status {
            TieredTrustDomainCheckpointStatus::Proposed => {
                return next == TieredTrustDomainCheckpointStatus::Approved
                    || next == TieredTrustDomainCheckpointStatus::Rejected;
            }
            TieredTrustDomainCheckpointStatus::Approved => {
                return next == TieredTrustDomainCheckpointStatus::Withdrawn;
            }
            TieredTrustDomainCheckpointStatus::Rejected => {
                return next == TieredTrustDomainCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTrustDomainCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered trust domain checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTrustDomainCheckpoint {
        TieredTrustDomainCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            contact_email: self.contact_email.clone(),
            budget: self.budget,
            retries: self.retries,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered trust domain checkpoint in status proposed.
pub fn new_tiered_trust_domain_checkpoint(code: &str, name: &str) -> TieredTrustDomainCheckpoint {
    TieredTrustDomainCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        starts_at: 0,
        updated_at: 0,
        contact_email: "".to_string(),
        budget: 0.0,
        retries: 0,
        ratio: 0.0,
        status: TieredTrustDomainCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_trust_domain_checkpoint("code-468", "name-606");
        assert_eq!(item.status, TieredTrustDomainCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_trust_domain_checkpoint("code-468", "name-606");
        if item.move_to(TieredTrustDomainCheckpointStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTrustDomainCheckpointStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_trust_domain_checkpoint("code-468", "name-606");
        item.status = TieredTrustDomainCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(TieredTrustDomainCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
