use super::*;

use acme_kit::text;

/// Where a trust domain estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TrustDomainEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain estimate as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainEstimate {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub owner_id: String,
    pub principal: String,
    pub amount: f64,
    pub contact_email: String,
    pub status: TrustDomainEstimateStatus,
}

impl TrustDomainEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainEstimateStatus) -> bool {
        match self.status {
            TrustDomainEstimateStatus::Draft => {
                match next {
                    TrustDomainEstimateStatus::Pending | TrustDomainEstimateStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEstimateStatus::Pending => {
                match next {
                    TrustDomainEstimateStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEstimateStatus::Active => {
                match next {
                    TrustDomainEstimateStatus::Suspended | TrustDomainEstimateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainEstimateStatus::Suspended => {
                match next {
                    TrustDomainEstimateStatus::Closed => {
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

    pub fn move_to(&mut self, next: TrustDomainEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TrustDomainEstimate {
        TrustDomainEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain estimate in status draft.
pub fn new_trust_domain_estimate(code: &str, label: &str) -> TrustDomainEstimate {
    TrustDomainEstimate {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        owner_id: "".to_string(),
        principal: "".to_string(),
        amount: 0.0,
        contact_email: "".to_string(),
        status: TrustDomainEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_estimate("code-205", "label-710");
        assert_eq!(item.status, TrustDomainEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_trust_domain_estimate("code-205", "label-710");
        if item.move_to(TrustDomainEstimateStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainEstimateStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_estimate("code-205", "label-710");
        item.status = TrustDomainEstimateStatus::Closed;
        assert!(
            item.move_to(TrustDomainEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
