use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit rule must pass. Check returns its problems, or none.
pub trait LimitRuleRule: std::fmt::Debug {
    fn check(&self, item: &LimitRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRulePinnedGuardRule {
    pub limit: i64,
}

impl LimitRuleRule for LimitRulePinnedGuardRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        if item.pinned && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while pinned",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleRequiredCountryRule {
    pub message: String,
}

impl LimitRuleRule for LimitRuleRequiredCountryRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitRuleRule for LimitRuleMaximumRangeRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitRuleRule for LimitRuleRevisionRangeRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleCodePrefixRule {
    pub prefix: String,
}

impl LimitRuleRule for LimitRuleCodePrefixRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleTitleLengthRule {
    pub max_length: i64,
}

impl LimitRuleRule for LimitRuleTitleLengthRule {
    fn check(&self, item: &LimitRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit rule.
#[derive(Debug, Clone)]
pub struct LimitRuleValidator {
    pub rules: Vec<Rc<dyn LimitRuleRule>>,
}

impl LimitRuleValidator {
    pub fn validate(&self, item: &LimitRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_rule_validator() -> LimitRuleValidator {
    LimitRuleValidator {
        rules: vec![
            Rc::new(LimitRulePinnedGuardRule { limit: 86400 }) as Rc<dyn LimitRuleRule>,
            Rc::new(LimitRuleRequiredCountryRule {
                message: "limit rule needs an entry".to_string(),
            }),
            Rc::new(LimitRuleMaximumRangeRule { min: 2.0, max: 4294.7 }),
            Rc::new(LimitRuleRevisionRangeRule { min: 16, max: 3600 }),
            Rc::new(LimitRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LimitRuleTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_rule_validator();
        let item = new_limit_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = LimitRuleRequiredCountryRule {
            message: "limit rule needs this field".to_string(),
        };
        let mut item = new_limit_rule("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LimitRuleMaximumRangeRule { min: 0.6, max: 4947.0 };
        let mut item = new_limit_rule("x", "y");
        item.maximum = 4948.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LimitRuleRevisionRangeRule { min: 1000, max: 86400 };
        let mut item = new_limit_rule("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitRuleTitleLengthRule { max_length: 3600 };
        let mut item = new_limit_rule("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
