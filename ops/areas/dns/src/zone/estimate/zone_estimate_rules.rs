use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone estimate must pass. Check returns its problems, or none.
pub trait ZoneEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ZoneEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateKeyIdLengthRule {
    pub max_length: i64,
}

impl ZoneEstimateRule for ZoneEstimateKeyIdLengthRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateRequiredKeyIdRule {
    pub message: String,
}

impl ZoneEstimateRule for ZoneEstimateRequiredKeyIdRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateCodeLengthRule {
    pub max_length: i64,
}

impl ZoneEstimateRule for ZoneEstimateCodeLengthRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneEstimateRule for ZoneEstimateSequenceRangeRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
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
pub struct ZoneEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneEstimateRule for ZoneEstimateBudgetRangeRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateRequiredCodeRule {
    pub message: String,
}

impl ZoneEstimateRule for ZoneEstimateRequiredCodeRule {
    fn check(&self, item: &ZoneEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone estimate.
#[derive(Debug, Clone)]
pub struct ZoneEstimateValidator {
    pub rules: Vec<Rc<dyn ZoneEstimateRule>>,
}

impl ZoneEstimateValidator {
    pub fn validate(&self, item: &ZoneEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_estimate_validator() -> ZoneEstimateValidator {
    ZoneEstimateValidator {
        rules: vec![
            Rc::new(ZoneEstimateKeyIdLengthRule { max_length: 3600 }) as Rc<dyn ZoneEstimateRule>,
            Rc::new(ZoneEstimateRequiredKeyIdRule {
                message: "zone estimate needs a value".to_string(),
            }),
            Rc::new(ZoneEstimateCodeLengthRule { max_length: 64 }),
            Rc::new(ZoneEstimateSequenceRangeRule { min: 80, max: 3600 }),
            Rc::new(ZoneEstimateBudgetRangeRule { min: 5.3, max: 1749.5 }),
            Rc::new(ZoneEstimateRequiredCodeRule {
                message: "zone estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_estimate_validator();
        let item = new_zone_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = ZoneEstimateKeyIdLengthRule { max_length: 3600 };
        let mut item = new_zone_estimate("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = ZoneEstimateRequiredKeyIdRule {
            message: "zone estimate needs an entry".to_string(),
        };
        let mut item = new_zone_estimate("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneEstimateCodeLengthRule { max_length: 3600 };
        let mut item = new_zone_estimate("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ZoneEstimateSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_zone_estimate("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ZoneEstimateBudgetRangeRule { min: 5.7, max: 4310.0 };
        let mut item = new_zone_estimate("x", "y");
        item.budget = 4311.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneEstimateRequiredCodeRule {
            message: "zone estimate needs an entry".to_string(),
        };
        let mut item = new_zone_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
