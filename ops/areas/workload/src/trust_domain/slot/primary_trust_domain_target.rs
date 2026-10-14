use super::*;

use acme_kit::text;

/// Where a primary trust domain target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTrustDomainTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryTrustDomainTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryTrustDomainTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary trust domain target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTarget {
    pub code: String,
    pub label: String,
    pub category: String,
    pub locale: String,
    pub cipher: String,
    pub contact_email: String,
    pub owner_id: String,
    pub external_ref: String,
    pub score: f64,
    pub status: PrimaryTrustDomainTargetStatus,
}

impl PrimaryTrustDomainTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTrustDomainTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTrustDomainTargetStatus) -> bool {
        match self.status {
            PrimaryTrustDomainTargetStatus::Created => {
                return next == PrimaryTrustDomainTargetStatus::Validated;
            }
            PrimaryTrustDomainTargetStatus::Validated => {
                return next == PrimaryTrustDomainTargetStatus::Retired;
            }
            PrimaryTrustDomainTargetStatus::Published => {
                match next {
                    PrimaryTrustDomainTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryTrustDomainTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary trust domain target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PrimaryTrustDomainTarget {
        PrimaryTrustDomainTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            locale: self.locale.clone(),
            cipher: self.cipher.clone(),
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            external_ref: self.external_ref.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary trust domain target in status created.
pub fn new_primary_trust_domain_target(code: &str, label: &str) -> PrimaryTrustDomainTarget {
    PrimaryTrustDomainTarget {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        locale: "".to_string(),
        cipher: "".to_string(),
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        external_ref: "".to_string(),
        score: 0.0,
        status: PrimaryTrustDomainTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_trust_domain_target("code-62", "label-693");
        assert_eq!(item.status, PrimaryTrustDomainTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_trust_domain_target("code-62", "label-693");
        if item.move_to(PrimaryTrustDomainTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTrustDomainTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_trust_domain_target("code-62", "label-693");
        item.status = PrimaryTrustDomainTargetStatus::Retired;
        assert!(
            item.move_to(PrimaryTrustDomainTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
