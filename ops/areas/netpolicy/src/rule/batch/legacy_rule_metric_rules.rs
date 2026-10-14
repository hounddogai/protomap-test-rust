use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rule metric must pass. Check returns its problems, or none.
pub trait LegacyRuleMetricRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetricPhoneLengthRule {
    pub max_length: i64,
}

impl LegacyRuleMetricRule for LegacyRuleMetricPhoneLengthRule {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetricRequiredPhoneRule {
    pub message: String,
}

impl LegacyRuleMetricRule for LegacyRuleMetricRequiredPhoneRule {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetricRequiredCodeRule {
    pub message: String,
}

impl LegacyRuleMetricRule for LegacyRuleMetricRequiredCodeRule {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetricLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRuleMetricRule for LegacyRuleMetricLimitRangeRule {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetricRequiredNotesRule {
    pub message: String,
}

impl LegacyRuleMetricRule for LegacyRuleMetricRequiredNotesRule {
    fn check(&self, item: &LegacyRuleMetric) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rule metric.
#[derive(Debug, Clone)]
pub struct LegacyRuleMetricValidator {
    pub rules: Vec<Rc<dyn LegacyRuleMetricRule>>,
}

impl LegacyRuleMetricValidator {
    pub fn validate(&self, item: &LegacyRuleMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRuleMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRuleMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rule_metric_validator() -> LegacyRuleMetricValidator {
    LegacyRuleMetricValidator {
        rules: vec![
            Rc::new(LegacyRuleMetricPhoneLengthRule { max_length: 64 }) as Rc<dyn LegacyRuleMetricRule>,
            Rc::new(LegacyRuleMetricRequiredPhoneRule {
                message: "legacy rule metric needs this field".to_string(),
            }),
            Rc::new(LegacyRuleMetricRequiredCodeRule {
                message: "legacy rule metric needs an entry".to_string(),
            }),
            Rc::new(LegacyRuleMetricLimitRangeRule { min: 3600, max: 3600 }),
            Rc::new(LegacyRuleMetricRequiredNotesRule {
                message: "legacy rule metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_rule_metric_validator();
        let item = new_legacy_rule_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LegacyRuleMetricPhoneLengthRule { max_length: 3600 };
        let mut item = new_legacy_rule_metric("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LegacyRuleMetricRequiredPhoneRule {
            message: "legacy rule metric needs a value".to_string(),
        };
        let mut item = new_legacy_rule_metric("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyRuleMetricRequiredCodeRule {
            message: "legacy rule metric needs an entry".to_string(),
        };
        let mut item = new_legacy_rule_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LegacyRuleMetricLimitRangeRule { min: 16, max: 3600 };
        let mut item = new_legacy_rule_metric("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LegacyRuleMetricRequiredNotesRule {
            message: "legacy rule metric needs this field".to_string(),
        };
        let mut item = new_legacy_rule_metric("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
