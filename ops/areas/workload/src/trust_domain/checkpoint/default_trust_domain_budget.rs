use super::*;

use acme_kit::text;

/// Where a default trust domain budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTrustDomainBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultTrustDomainBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultTrustDomainBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default trust domain budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainBudget {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub port: i64,
    pub reviewer: String,
    pub budget: f64,
    pub phone: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub archived: bool,
    pub status: DefaultTrustDomainBudgetStatus,
}

impl DefaultTrustDomainBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTrustDomainBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTrustDomainBudgetStatus) -> bool {
        match self.status {
            DefaultTrustDomainBudgetStatus::Requested => {
                return next == DefaultTrustDomainBudgetStatus::Granted
                    || next == DefaultTrustDomainBudgetStatus::Expired;
            }
            DefaultTrustDomainBudgetStatus::Granted => {
                match next {
                    DefaultTrustDomainBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTrustDomainBudgetStatus::Revoked => {
                match next {
                    DefaultTrustDomainBudgetStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultTrustDomainBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default trust domain budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.budget * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTrustDomainBudget {
        DefaultTrustDomainBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            port: self.port,
            reviewer: self.reviewer.clone(),
            budget: self.budget,
            phone: self.phone.clone(),
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default trust domain budget in status requested.
pub fn new_default_trust_domain_budget(code: &str, label: &str) -> DefaultTrustDomainBudget {
    DefaultTrustDomainBudget {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        port: 0,
        reviewer: "".to_string(),
        budget: 0.0,
        phone: "".to_string(),
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        archived: false,
        status: DefaultTrustDomainBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_trust_domain_budget("code-30", "label-262");
        assert_eq!(item.status, DefaultTrustDomainBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_trust_domain_budget("code-30", "label-262");
        if item.move_to(DefaultTrustDomainBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTrustDomainBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_trust_domain_budget("code-30", "label-262");
        item.status = DefaultTrustDomainBudgetStatus::Expired;
        assert!(
            item.move_to(DefaultTrustDomainBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
