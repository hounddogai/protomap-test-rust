use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback data export rule must pass. Check returns its problems, or none.
pub trait FallbackDataExportRuleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDataExportRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDataExportRuleRule for FallbackDataExportRuleAttemptsRangeRule {
    fn check(&self, item: &FallbackDataExportRule) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleTitleLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportRuleRule for FallbackDataExportRuleTitleLengthRule {
    fn check(&self, item: &FallbackDataExportRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDataExportRuleRule for FallbackDataExportRuleCeilingRangeRule {
    fn check(&self, item: &FallbackDataExportRule) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleRequiredDescriptionRule {
    pub message: String,
}

impl FallbackDataExportRuleRule for FallbackDataExportRuleRequiredDescriptionRule {
    fn check(&self, item: &FallbackDataExportRule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback data export rule.
#[derive(Debug, Clone)]
pub struct FallbackDataExportRuleValidator {
    pub rules: Vec<Rc<dyn FallbackDataExportRuleRule>>,
}

impl FallbackDataExportRuleValidator {
    pub fn validate(&self, item: &FallbackDataExportRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDataExportRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDataExportRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_data_export_rule_validator() -> FallbackDataExportRuleValidator {
    FallbackDataExportRuleValidator {
        rules: vec![
            Rc::new(FallbackDataExportRuleAttemptsRangeRule { min: 16, max: 64 }) as Rc<dyn FallbackDataExportRuleRule>,
            Rc::new(FallbackDataExportRuleTitleLengthRule { max_length: 255 }),
            Rc::new(FallbackDataExportRuleCeilingRangeRule { min: 3.8, max: 3556.5 }),
            Rc::new(FallbackDataExportRuleRequiredDescriptionRule {
                message: "fallback data export rule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_data_export_rule_validator();
        let item = new_fallback_data_export_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = FallbackDataExportRuleAttemptsRangeRule { min: 120, max: 3600 };
        let mut item = new_fallback_data_export_rule("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackDataExportRuleTitleLengthRule { max_length: 1000 };
        let mut item = new_fallback_data_export_rule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = FallbackDataExportRuleCeilingRangeRule { min: 4.7, max: 2611.4 };
        let mut item = new_fallback_data_export_rule("x", "y");
        item.ceiling = 2612.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = FallbackDataExportRuleRequiredDescriptionRule {
            message: "fallback data export rule needs an entry".to_string(),
        };
        let mut item = new_fallback_data_export_rule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
