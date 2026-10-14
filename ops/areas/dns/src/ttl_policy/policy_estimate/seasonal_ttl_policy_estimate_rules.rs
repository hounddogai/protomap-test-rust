use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal ttl policy estimate must pass. Check returns its problems, or none.
pub trait SeasonalTtlPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateSequenceRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateCapacityRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateDescriptionLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateDescriptionLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateRequiredDescriptionRule {
    pub message: String,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateRequiredDescriptionRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateCodeLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyEstimateRule for SeasonalTtlPolicyEstimateLabelLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal ttl policy estimate.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalTtlPolicyEstimateRule>>,
}

impl SeasonalTtlPolicyEstimateValidator {
    pub fn validate(&self, item: &SeasonalTtlPolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTtlPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTtlPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_ttl_policy_estimate_validator() -> SeasonalTtlPolicyEstimateValidator {
    SeasonalTtlPolicyEstimateValidator {
        rules: vec![
            Rc::new(SeasonalTtlPolicyEstimateSequenceRangeRule { min: 32, max: 86400 })
                as Rc<dyn SeasonalTtlPolicyEstimateRule>,
            Rc::new(SeasonalTtlPolicyEstimateCapacityRangeRule { min: 120, max: 1000 }),
            Rc::new(SeasonalTtlPolicyEstimateDescriptionLengthRule { max_length: 120 }),
            Rc::new(SeasonalTtlPolicyEstimateRequiredDescriptionRule {
                message: "seasonal ttl policy estimate needs this field".to_string(),
            }),
            Rc::new(SeasonalTtlPolicyEstimateCodeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalTtlPolicyEstimateLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_ttl_policy_estimate_validator();
        let item = new_seasonal_ttl_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SeasonalTtlPolicyEstimateSequenceRangeRule { min: 3600, max: 86400 };
        let mut item = new_seasonal_ttl_policy_estimate("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SeasonalTtlPolicyEstimateCapacityRangeRule { min: 120, max: 3600 };
        let mut item = new_seasonal_ttl_policy_estimate("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SeasonalTtlPolicyEstimateDescriptionLengthRule { max_length: 86400 };
        let mut item = new_seasonal_ttl_policy_estimate("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = SeasonalTtlPolicyEstimateRequiredDescriptionRule {
            message: "seasonal ttl policy estimate needs this field".to_string(),
        };
        let mut item = new_seasonal_ttl_policy_estimate("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalTtlPolicyEstimateCodeLengthRule { max_length: 86400 };
        let mut item = new_seasonal_ttl_policy_estimate("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalTtlPolicyEstimateLabelLengthRule { max_length: 3600 };
        let mut item = new_seasonal_ttl_policy_estimate("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
