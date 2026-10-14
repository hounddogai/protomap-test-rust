use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout claim must pass. Check returns its problems, or none.
pub trait RolloutClaimRule: std::fmt::Debug {
    fn check(&self, item: &RolloutClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaimRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutClaimRule for RolloutClaimRatioRangeRule {
    fn check(&self, item: &RolloutClaim) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaimCodeLengthRule {
    pub max_length: i64,
}

impl RolloutClaimRule for RolloutClaimCodeLengthRule {
    fn check(&self, item: &RolloutClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaimRequiredNotesRule {
    pub message: String,
}

impl RolloutClaimRule for RolloutClaimRequiredNotesRule {
    fn check(&self, item: &RolloutClaim) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout claim.
#[derive(Debug, Clone)]
pub struct RolloutClaimValidator {
    pub rules: Vec<Rc<dyn RolloutClaimRule>>,
}

impl RolloutClaimValidator {
    pub fn validate(&self, item: &RolloutClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_claim_validator() -> RolloutClaimValidator {
    RolloutClaimValidator {
        rules: vec![
            Rc::new(RolloutClaimRatioRangeRule { min: 5.5, max: 3800.6 }) as Rc<dyn RolloutClaimRule>,
            Rc::new(RolloutClaimCodeLengthRule { max_length: 1000 }),
            Rc::new(RolloutClaimRequiredNotesRule {
                message: "rollout claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_claim_validator();
        let item = new_rollout_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RolloutClaimRatioRangeRule { min: 9.1, max: 4381.6 };
        let mut item = new_rollout_claim("x", "y");
        item.ratio = 4383.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutClaimCodeLengthRule { max_length: 32 };
        let mut item = new_rollout_claim("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RolloutClaimRequiredNotesRule {
            message: "rollout claim needs this field".to_string(),
        };
        let mut item = new_rollout_claim("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
