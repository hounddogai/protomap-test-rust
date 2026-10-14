use super::*;

use acme_kit::text;

/// Where a lenient health check rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientHealthCheckRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientHealthCheckRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientHealthCheckRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient health check rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRule {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub currency: String,
    pub window_days: i64,
    pub score: f64,
    pub maximum: f64,
    pub port: i64,
    pub tags: Vec<String>,
    pub status: LenientHealthCheckRuleStatus,
}

impl LenientHealthCheckRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientHealthCheckRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientHealthCheckRuleStatus) -> bool {
        match self.status {
            LenientHealthCheckRuleStatus::Requested => {
                return next == LenientHealthCheckRuleStatus::Granted || next == LenientHealthCheckRuleStatus::Expired;
            }
            LenientHealthCheckRuleStatus::Granted => {
                return next == LenientHealthCheckRuleStatus::Revoked || next == LenientHealthCheckRuleStatus::Expired;
            }
            LenientHealthCheckRuleStatus::Revoked => {
                match next {
                    LenientHealthCheckRuleStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientHealthCheckRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient health check rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.score - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientHealthCheckRule {
        LenientHealthCheckRule {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            window_days: self.window_days,
            score: self.score,
            maximum: self.maximum,
            port: self.port,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient health check rule in status requested.
pub fn new_lenient_health_check_rule(code: &str, name: &str) -> LenientHealthCheckRule {
    LenientHealthCheckRule {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        window_days: 0,
        score: 0.0,
        maximum: 0.0,
        port: 0,
        tags: Vec::new(),
        status: LenientHealthCheckRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_health_check_rule("code-819", "name-153");
        assert_eq!(item.status, LenientHealthCheckRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_lenient_health_check_rule("code-819", "name-153");
        if item.move_to(LenientHealthCheckRuleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientHealthCheckRuleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_health_check_rule("code-819", "name-153");
        item.status = LenientHealthCheckRuleStatus::Expired;
        assert!(
            item.move_to(LenientHealthCheckRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
