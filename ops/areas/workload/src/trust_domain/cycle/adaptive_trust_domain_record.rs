use super::*;

use acme_kit::text;

/// Where a adaptive trust domain record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTrustDomainRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveTrustDomainRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveTrustDomainRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive trust domain record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecord {
    pub code: String,
    pub title: String,
    pub category: String,
    pub minimum: f64,
    pub locale: String,
    pub checksum: String,
    pub summary: String,
    pub status: AdaptiveTrustDomainRecordStatus,
}

impl AdaptiveTrustDomainRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTrustDomainRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTrustDomainRecordStatus) -> bool {
        match self.status {
            AdaptiveTrustDomainRecordStatus::Proposed => {
                return next == AdaptiveTrustDomainRecordStatus::Approved
                    || next == AdaptiveTrustDomainRecordStatus::Withdrawn;
            }
            AdaptiveTrustDomainRecordStatus::Approved => {
                return next == AdaptiveTrustDomainRecordStatus::Rejected;
            }
            AdaptiveTrustDomainRecordStatus::Rejected => {
                return next == AdaptiveTrustDomainRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTrustDomainRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive trust domain record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> AdaptiveTrustDomainRecord {
        AdaptiveTrustDomainRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            minimum: self.minimum,
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive trust domain record in status proposed.
pub fn new_adaptive_trust_domain_record(code: &str, title: &str) -> AdaptiveTrustDomainRecord {
    AdaptiveTrustDomainRecord {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        minimum: 0.0,
        locale: "".to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        status: AdaptiveTrustDomainRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_trust_domain_record("code-591", "title-405");
        assert_eq!(item.status, AdaptiveTrustDomainRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_adaptive_trust_domain_record("code-591", "title-405");
        if item.move_to(AdaptiveTrustDomainRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTrustDomainRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_trust_domain_record("code-591", "title-405");
        item.status = AdaptiveTrustDomainRecordStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveTrustDomainRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
