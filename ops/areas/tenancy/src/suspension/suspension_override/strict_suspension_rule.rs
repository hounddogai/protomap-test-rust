use super::*;

use acme_kit::text;

/// Where a strict suspension rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSuspensionRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictSuspensionRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictSuspensionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict suspension rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRule {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub budget: f64,
    pub archived: bool,
    pub source: String,
    pub category: String,
    pub expires_at: i64,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub weight: f64,
    pub status: StrictSuspensionRuleStatus,
}

impl StrictSuspensionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSuspensionRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSuspensionRuleStatus) -> bool {
        match self.status {
            StrictSuspensionRuleStatus::Requested => {
                return next == StrictSuspensionRuleStatus::Revoked || next == StrictSuspensionRuleStatus::Expired;
            }
            StrictSuspensionRuleStatus::Granted => {
                return next == StrictSuspensionRuleStatus::Revoked || next == StrictSuspensionRuleStatus::Expired;
            }
            StrictSuspensionRuleStatus::Revoked => {
                match next {
                    StrictSuspensionRuleStatus::Expired => {
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

    pub fn move_to(&mut self, next: StrictSuspensionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict suspension rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict suspension rule", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.expires_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSuspensionRule {
        StrictSuspensionRule {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            budget: self.budget,
            archived: self.archived,
            source: self.source.clone(),
            category: self.category.clone(),
            expires_at: self.expires_at,
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict suspension rule in status requested.
pub fn new_strict_suspension_rule(code: &str, name: &str) -> StrictSuspensionRule {
    StrictSuspensionRule {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        budget: 0.0,
        archived: false,
        source: "".to_string(),
        category: "".to_string(),
        expires_at: 0,
        attempts: 0,
        ttl_seconds: 0,
        weight: 0.0,
        status: StrictSuspensionRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_suspension_rule("code-513", "name-896");
        assert_eq!(item.status, StrictSuspensionRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_strict_suspension_rule("code-513", "name-896");
        if item.move_to(StrictSuspensionRuleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSuspensionRuleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_suspension_rule("code-513", "name-896");
        item.status = StrictSuspensionRuleStatus::Expired;
        assert!(
            item.move_to(StrictSuspensionRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
