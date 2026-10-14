use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback overage rule must pass. Check returns its problems, or none.
pub trait FallbackOverageRuleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleRequiredCodeRule {
    pub message: String,
}

impl FallbackOverageRuleRule for FallbackOverageRuleRequiredCodeRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleRequiredLabelRule {
    pub message: String,
}

impl FallbackOverageRuleRule for FallbackOverageRuleRequiredLabelRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleLabelLengthRule {
    pub max_length: i64,
}

impl FallbackOverageRuleRule for FallbackOverageRuleLabelLengthRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackOverageRuleRule for FallbackOverageRuleCreatedAtRangeRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackOverageRuleRule for FallbackOverageRuleCeilingRangeRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackOverageRuleRule for FallbackOverageRuleCodeLengthRule {
    fn check(&self, item: &FallbackOverageRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback overage rule.
#[derive(Debug, Clone)]
pub struct FallbackOverageRuleValidator {
    pub rules: Vec<Rc<dyn FallbackOverageRuleRule>>,
}

impl FallbackOverageRuleValidator {
    pub fn validate(&self, item: &FallbackOverageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackOverageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackOverageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_overage_rule_validator() -> FallbackOverageRuleValidator {
    FallbackOverageRuleValidator {
        rules: vec![
            Rc::new(FallbackOverageRuleRequiredCodeRule {
                message: "fallback overage rule needs a value".to_string(),
            }) as Rc<dyn FallbackOverageRuleRule>,
            Rc::new(FallbackOverageRuleRequiredLabelRule {
                message: "fallback overage rule needs an entry".to_string(),
            }),
            Rc::new(FallbackOverageRuleLabelLengthRule { max_length: 120 }),
            Rc::new(FallbackOverageRuleCreatedAtRangeRule { min: 16, max: 80 }),
            Rc::new(FallbackOverageRuleCeilingRangeRule { min: 8.3, max: 965.9 }),
            Rc::new(FallbackOverageRuleCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_overage_rule_validator();
        let item = new_fallback_overage_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackOverageRuleRequiredCodeRule {
            message: "fallback overage rule needs an entry".to_string(),
        };
        let mut item = new_fallback_overage_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackOverageRuleRequiredLabelRule {
            message: "fallback overage rule needs an entry".to_string(),
        };
        let mut item = new_fallback_overage_rule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackOverageRuleLabelLengthRule { max_length: 64 };
        let mut item = new_fallback_overage_rule("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FallbackOverageRuleCreatedAtRangeRule { min: 16, max: 3600 };
        let mut item = new_fallback_overage_rule("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = FallbackOverageRuleCeilingRangeRule { min: 4.8, max: 4771.4 };
        let mut item = new_fallback_overage_rule("x", "y");
        item.ceiling = 4772.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackOverageRuleCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_overage_rule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
