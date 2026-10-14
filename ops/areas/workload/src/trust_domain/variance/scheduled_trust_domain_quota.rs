use super::*;

use acme_kit::text;

/// Where a scheduled trust domain quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTrustDomainQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledTrustDomainQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledTrustDomainQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled trust domain quota as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainQuota {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub rate: f64,
    pub external_ref: String,
    pub discount: f64,
    pub status: ScheduledTrustDomainQuotaStatus,
}

impl ScheduledTrustDomainQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTrustDomainQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTrustDomainQuotaStatus) -> bool {
        match self.status {
            ScheduledTrustDomainQuotaStatus::Proposed => {
                return next == ScheduledTrustDomainQuotaStatus::Rejected
                    || next == ScheduledTrustDomainQuotaStatus::Withdrawn;
            }
            ScheduledTrustDomainQuotaStatus::Approved => {
                match next {
                    ScheduledTrustDomainQuotaStatus::Rejected | ScheduledTrustDomainQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTrustDomainQuotaStatus::Rejected => {
                return next == ScheduledTrustDomainQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTrustDomainQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled trust domain quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rate - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTrustDomainQuota {
        ScheduledTrustDomainQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled trust domain quota in status proposed.
pub fn new_scheduled_trust_domain_quota(code: &str, label: &str) -> ScheduledTrustDomainQuota {
    ScheduledTrustDomainQuota {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        rate: 0.0,
        external_ref: "".to_string(),
        discount: 0.0,
        status: ScheduledTrustDomainQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_trust_domain_quota("code-957", "label-59");
        assert_eq!(item.status, ScheduledTrustDomainQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_scheduled_trust_domain_quota("code-957", "label-59");
        if item.move_to(ScheduledTrustDomainQuotaStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTrustDomainQuotaStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_trust_domain_quota("code-957", "label-59");
        item.status = ScheduledTrustDomainQuotaStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledTrustDomainQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
