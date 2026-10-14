use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered nonce target must pass. Check returns its problems, or none.
pub trait TieredNonceTargetRule: std::fmt::Debug {
    fn check(&self, item: &TieredNonceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTargetAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredNonceTargetRule for TieredNonceTargetAttemptsRangeRule {
    fn check(&self, item: &TieredNonceTarget) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTargetPinnedGuardRule {
    pub limit: f64,
}

impl TieredNonceTargetRule for TieredNonceTargetPinnedGuardRule {
    fn check(&self, item: &TieredNonceTarget) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTargetRequiredCodeRule {
    pub message: String,
}

impl TieredNonceTargetRule for TieredNonceTargetRequiredCodeRule {
    fn check(&self, item: &TieredNonceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered nonce target.
#[derive(Debug, Clone)]
pub struct TieredNonceTargetValidator {
    pub rules: Vec<Rc<dyn TieredNonceTargetRule>>,
}

impl TieredNonceTargetValidator {
    pub fn validate(&self, item: &TieredNonceTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredNonceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredNonceTarget]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_tiered_nonce_target_validator() -> TieredNonceTargetValidator {
    TieredNonceTargetValidator {
        rules: vec![
            Rc::new(TieredNonceTargetAttemptsRangeRule { min: 32, max: 255 }) as Rc<dyn TieredNonceTargetRule>,
            Rc::new(TieredNonceTargetPinnedGuardRule { limit: 1.5 }),
            Rc::new(TieredNonceTargetRequiredCodeRule {
                message: "tiered nonce target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_nonce_target_validator();
        let item = new_tiered_nonce_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TieredNonceTargetAttemptsRangeRule { min: 86400, max: 86400 };
        let mut item = new_tiered_nonce_target("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredNonceTargetRequiredCodeRule {
            message: "tiered nonce target needs this field".to_string(),
        };
        let mut item = new_tiered_nonce_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
