use super::*;

use acme_kit::text;

/// Where a rule claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaim {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub country: String,
    pub threshold: f64,
    pub grace_days: i64,
    pub ceiling: f64,
    pub phone: String,
    pub retries: i64,
    pub updated_at: i64,
    pub status: RuleClaimStatus,
}

impl RuleClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleClaimStatus) -> bool {
        match self.status {
            RuleClaimStatus::Requested => {
                return next == RuleClaimStatus::Revoked;
            }
            RuleClaimStatus::Granted => {
                match next {
                    RuleClaimStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleClaimStatus::Revoked => {
                return next == RuleClaimStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ceiling + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleClaim {
        RuleClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            country: self.country.clone(),
            threshold: self.threshold,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            phone: self.phone.clone(),
            retries: self.retries,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rule claim in status requested.
pub fn new_rule_claim(code: &str, label: &str) -> RuleClaim {
    RuleClaim {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        country: "".to_string(),
        threshold: 0.0,
        grace_days: 0,
        ceiling: 0.0,
        phone: "".to_string(),
        retries: 0,
        updated_at: 0,
        status: RuleClaimStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_claim("code-506", "label-735");
        assert_eq!(item.status, RuleClaimStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rule_claim("code-506", "label-735");
        if item.move_to(RuleClaimStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleClaimStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_claim("code-506", "label-735");
        item.status = RuleClaimStatus::Expired;
        assert!(
            item.move_to(RuleClaimStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
