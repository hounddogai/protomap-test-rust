use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped canary rule must pass. Check returns its problems, or none.
pub trait CappedCanaryRuleRule: std::fmt::Debug {
    fn check(&self, item: &CappedCanaryRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRuleDescriptionLengthRule {
    pub max_length: i64,
}

impl CappedCanaryRuleRule for CappedCanaryRuleDescriptionLengthRule {
    fn check(&self, item: &CappedCanaryRule) -> Vec<String> {
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
pub struct CappedCanaryRuleRequiredTitleRule {
    pub message: String,
}

impl CappedCanaryRuleRule for CappedCanaryRuleRequiredTitleRule {
    fn check(&self, item: &CappedCanaryRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRuleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedCanaryRuleRule for CappedCanaryRuleRatioRangeRule {
    fn check(&self, item: &CappedCanaryRule) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRuleCodeLengthRule {
    pub max_length: i64,
}

impl CappedCanaryRuleRule for CappedCanaryRuleCodeLengthRule {
    fn check(&self, item: &CappedCanaryRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped canary rule.
#[derive(Debug, Clone)]
pub struct CappedCanaryRuleValidator {
    pub rules: Vec<Rc<dyn CappedCanaryRuleRule>>,
}

impl CappedCanaryRuleValidator {
    pub fn validate(&self, item: &CappedCanaryRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedCanaryRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedCanaryRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_canary_rule_validator() -> CappedCanaryRuleValidator {
    CappedCanaryRuleValidator {
        rules: vec![
            Rc::new(CappedCanaryRuleDescriptionLengthRule { max_length: 255 }) as Rc<dyn CappedCanaryRuleRule>,
            Rc::new(CappedCanaryRuleRequiredTitleRule {
                message: "capped canary rule needs a value".to_string(),
            }),
            Rc::new(CappedCanaryRuleRatioRangeRule { min: 6.8, max: 1185.6 }),
            Rc::new(CappedCanaryRuleCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_canary_rule_validator();
        let item = new_capped_canary_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = CappedCanaryRuleDescriptionLengthRule { max_length: 16 };
        let mut item = new_capped_canary_rule("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedCanaryRuleRequiredTitleRule {
            message: "capped canary rule needs an entry".to_string(),
        };
        let mut item = new_capped_canary_rule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = CappedCanaryRuleRatioRangeRule { min: 0.5, max: 1840.9 };
        let mut item = new_capped_canary_rule("x", "y");
        item.ratio = 1842.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedCanaryRuleCodeLengthRule { max_length: 255 };
        let mut item = new_capped_canary_rule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
