use super::*;

use acme_kit::text;

/// Where a fallback delegation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDelegationEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackDelegationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackDelegationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback delegation estimate as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationEstimate {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub revision: i64,
    pub description: String,
    pub status: FallbackDelegationEstimateStatus,
}

impl FallbackDelegationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDelegationEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDelegationEstimateStatus) -> bool {
        match self.status {
            FallbackDelegationEstimateStatus::Requested => {
                return next == FallbackDelegationEstimateStatus::Revoked;
            }
            FallbackDelegationEstimateStatus::Granted => {
                return next == FallbackDelegationEstimateStatus::Expired;
            }
            FallbackDelegationEstimateStatus::Revoked => {
                match next {
                    FallbackDelegationEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackDelegationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback delegation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback delegation estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> FallbackDelegationEstimate {
        FallbackDelegationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            revision: self.revision,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback delegation estimate in status requested.
pub fn new_fallback_delegation_estimate(code: &str, label: &str) -> FallbackDelegationEstimate {
    FallbackDelegationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        revision: 0,
        description: "".to_string(),
        status: FallbackDelegationEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_delegation_estimate("code-674", "label-531");
        assert_eq!(item.status, FallbackDelegationEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_delegation_estimate("code-674", "label-531");
        if item.move_to(FallbackDelegationEstimateStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDelegationEstimateStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_delegation_estimate("code-674", "label-531");
        item.status = FallbackDelegationEstimateStatus::Expired;
        assert!(
            item.move_to(FallbackDelegationEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
