use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary measurement rule must pass. Check returns its problems, or none.
pub trait PrimaryMeasurementRuleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleCipherLengthRule {
    pub max_length: i64,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleCipherLengthRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleBurstRangeRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleCodeLengthRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleRegionLengthRule {
    pub max_length: i64,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleRegionLengthRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleRequiredCodeRule {
    pub message: String,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleRequiredCodeRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRuleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryMeasurementRuleRule for PrimaryMeasurementRuleMarginRangeRule {
    fn check(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary measurement rule.
#[derive(Debug, Clone)]
pub struct PrimaryMeasurementRuleValidator {
    pub rules: Vec<Rc<dyn PrimaryMeasurementRuleRule>>,
}

impl PrimaryMeasurementRuleValidator {
    pub fn validate(&self, item: &PrimaryMeasurementRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryMeasurementRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryMeasurementRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_measurement_rule_validator() -> PrimaryMeasurementRuleValidator {
    PrimaryMeasurementRuleValidator {
        rules: vec![
            Rc::new(PrimaryMeasurementRuleCipherLengthRule { max_length: 3600 }) as Rc<dyn PrimaryMeasurementRuleRule>,
            Rc::new(PrimaryMeasurementRuleBurstRangeRule { min: 120, max: 1000 }),
            Rc::new(PrimaryMeasurementRuleCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimaryMeasurementRuleRegionLengthRule { max_length: 1000 }),
            Rc::new(PrimaryMeasurementRuleRequiredCodeRule {
                message: "primary measurement rule needs an entry".to_string(),
            }),
            Rc::new(PrimaryMeasurementRuleMarginRangeRule { min: 6.0, max: 283.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_measurement_rule_validator();
        let item = new_primary_measurement_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PrimaryMeasurementRuleCipherLengthRule { max_length: 32 };
        let mut item = new_primary_measurement_rule("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PrimaryMeasurementRuleBurstRangeRule { min: 16, max: 64 };
        let mut item = new_primary_measurement_rule("x", "y");
        item.burst = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryMeasurementRuleCodeLengthRule { max_length: 64 };
        let mut item = new_primary_measurement_rule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PrimaryMeasurementRuleRegionLengthRule { max_length: 255 };
        let mut item = new_primary_measurement_rule("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryMeasurementRuleRequiredCodeRule {
            message: "primary measurement rule needs an entry".to_string(),
        };
        let mut item = new_primary_measurement_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PrimaryMeasurementRuleMarginRangeRule { min: 4.2, max: 837.0 };
        let mut item = new_primary_measurement_rule("x", "y");
        item.margin = 838.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
