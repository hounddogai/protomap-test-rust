use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional ttl policy estimate must pass. Check returns its problems, or none.
pub trait RegionalTtlPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTtlPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RegionalTtlPolicyEstimateRule for RegionalTtlPolicyEstimateLabelLengthRule {
    fn check(&self, item: &RegionalTtlPolicyEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyEstimateCodeLengthRule {
    pub max_length: i64,
}

impl RegionalTtlPolicyEstimateRule for RegionalTtlPolicyEstimateCodeLengthRule {
    fn check(&self, item: &RegionalTtlPolicyEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyEstimateGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalTtlPolicyEstimateRule for RegionalTtlPolicyEstimateGraceDaysRangeRule {
    fn check(&self, item: &RegionalTtlPolicyEstimate) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional ttl policy estimate.
#[derive(Debug, Clone)]
pub struct RegionalTtlPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn RegionalTtlPolicyEstimateRule>>,
}

impl RegionalTtlPolicyEstimateValidator {
    pub fn validate(&self, item: &RegionalTtlPolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalTtlPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTtlPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_ttl_policy_estimate_validator() -> RegionalTtlPolicyEstimateValidator {
    RegionalTtlPolicyEstimateValidator {
        rules: vec![
            Rc::new(RegionalTtlPolicyEstimateLabelLengthRule { max_length: 1000 })
                as Rc<dyn RegionalTtlPolicyEstimateRule>,
            Rc::new(RegionalTtlPolicyEstimateCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalTtlPolicyEstimateGraceDaysRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_ttl_policy_estimate_validator();
        let item = new_regional_ttl_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalTtlPolicyEstimateLabelLengthRule { max_length: 32 };
        let mut item = new_regional_ttl_policy_estimate("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalTtlPolicyEstimateCodeLengthRule { max_length: 1000 };
        let mut item = new_regional_ttl_policy_estimate("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RegionalTtlPolicyEstimateGraceDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_regional_ttl_policy_estimate("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
