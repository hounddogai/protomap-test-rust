use super::*;

use acme_kit::text;

/// Where a default policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPolicyEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultPolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default policy estimate as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyEstimate {
    pub code: String,
    pub title: String,
    pub name: String,
    pub contact_email: String,
    pub sequence: i64,
    pub discount: f64,
    pub status: DefaultPolicyEstimateStatus,
}

impl DefaultPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPolicyEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPolicyEstimateStatus) -> bool {
        match self.status {
            DefaultPolicyEstimateStatus::Requested => {
                return next == DefaultPolicyEstimateStatus::Granted || next == DefaultPolicyEstimateStatus::Revoked;
            }
            DefaultPolicyEstimateStatus::Granted => {
                match next {
                    DefaultPolicyEstimateStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPolicyEstimateStatus::Revoked => {
                return next == DefaultPolicyEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.discount + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPolicyEstimate {
        DefaultPolicyEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default policy estimate in status requested.
pub fn new_default_policy_estimate(code: &str, title: &str) -> DefaultPolicyEstimate {
    DefaultPolicyEstimate {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        contact_email: "".to_string(),
        sequence: 0,
        discount: 0.0,
        status: DefaultPolicyEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_policy_estimate("code-318", "title-600");
        assert_eq!(item.status, DefaultPolicyEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_policy_estimate("code-318", "title-600");
        if item.move_to(DefaultPolicyEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPolicyEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_policy_estimate("code-318", "title-600");
        item.status = DefaultPolicyEstimateStatus::Expired;
        assert!(
            item.move_to(DefaultPolicyEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
