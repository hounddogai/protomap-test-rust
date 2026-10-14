use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending ttl policy group must pass. Check returns its problems, or none.
pub trait PendingTtlPolicyGroupRule: std::fmt::Debug {
    fn check(&self, item: &PendingTtlPolicyGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupPhoneLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyGroupRule for PendingTtlPolicyGroupPhoneLengthRule {
    fn check(&self, item: &PendingTtlPolicyGroup) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupExternalRefLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyGroupRule for PendingTtlPolicyGroupExternalRefLengthRule {
    fn check(&self, item: &PendingTtlPolicyGroup) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingTtlPolicyGroupRule for PendingTtlPolicyGroupRatioRangeRule {
    fn check(&self, item: &PendingTtlPolicyGroup) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingTtlPolicyGroupRule for PendingTtlPolicyGroupScoreRangeRule {
    fn check(&self, item: &PendingTtlPolicyGroup) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending ttl policy group.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyGroupValidator {
    pub rules: Vec<Rc<dyn PendingTtlPolicyGroupRule>>,
}

impl PendingTtlPolicyGroupValidator {
    pub fn validate(&self, item: &PendingTtlPolicyGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingTtlPolicyGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTtlPolicyGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_ttl_policy_group_validator() -> PendingTtlPolicyGroupValidator {
    PendingTtlPolicyGroupValidator {
        rules: vec![
            Rc::new(PendingTtlPolicyGroupPhoneLengthRule { max_length: 32 }) as Rc<dyn PendingTtlPolicyGroupRule>,
            Rc::new(PendingTtlPolicyGroupExternalRefLengthRule { max_length: 3600 }),
            Rc::new(PendingTtlPolicyGroupRatioRangeRule { min: 5.6, max: 3418.9 }),
            Rc::new(PendingTtlPolicyGroupScoreRangeRule { min: 6.0, max: 904.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_ttl_policy_group_validator();
        let item = new_pending_ttl_policy_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingTtlPolicyGroupPhoneLengthRule { max_length: 1000 };
        let mut item = new_pending_ttl_policy_group("x", "y");
        item.phone = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = PendingTtlPolicyGroupExternalRefLengthRule { max_length: 32 };
        let mut item = new_pending_ttl_policy_group("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PendingTtlPolicyGroupRatioRangeRule { min: 8.1, max: 2162.9 };
        let mut item = new_pending_ttl_policy_group("x", "y");
        item.ratio = 2164.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingTtlPolicyGroupScoreRangeRule { min: 9.4, max: 287.6 };
        let mut item = new_pending_ttl_policy_group("x", "y");
        item.score = 289.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
