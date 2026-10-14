use super::*;

use acme_kit::text;

/// Where a rule estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule estimate as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleEstimate {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub status: RuleEstimateStatus,
}

impl RuleEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleEstimateStatus) -> bool {
        match self.status {
            RuleEstimateStatus::Requested => {
                match next {
                    RuleEstimateStatus::Revoked | RuleEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleEstimateStatus::Granted => {
                match next {
                    RuleEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleEstimateStatus::Revoked => {
                match next {
                    RuleEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: RuleEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleEstimate {
        RuleEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rule estimate in status requested.
pub fn new_rule_estimate(code: &str, label: &str) -> RuleEstimate {
    RuleEstimate {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        rotation_days: 0,
        contact_email: "".to_string(),
        ttl_seconds: 0,
        status: RuleEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_estimate("code-79", "label-261");
        assert_eq!(item.status, RuleEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rule_estimate("code-79", "label-261");
        if item.move_to(RuleEstimateStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleEstimateStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_estimate("code-79", "label-261");
        item.status = RuleEstimateStatus::Expired;
        assert!(
            item.move_to(RuleEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
