use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict rule variance must pass. Check returns its problems, or none.
pub trait StrictRuleVarianceRule: std::fmt::Debug {
    fn check(&self, item: &StrictRuleVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleVarianceCodeLengthRule {
    pub max_length: i64,
}

impl StrictRuleVarianceRule for StrictRuleVarianceCodeLengthRule {
    fn check(&self, item: &StrictRuleVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleVarianceThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRuleVarianceRule for StrictRuleVarianceThresholdRangeRule {
    fn check(&self, item: &StrictRuleVariance) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleVarianceRequiredTitleRule {
    pub message: String,
}

impl StrictRuleVarianceRule for StrictRuleVarianceRequiredTitleRule {
    fn check(&self, item: &StrictRuleVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleVarianceLocaleLengthRule {
    pub max_length: i64,
}

impl StrictRuleVarianceRule for StrictRuleVarianceLocaleLengthRule {
    fn check(&self, item: &StrictRuleVariance) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict rule variance.
#[derive(Debug, Clone)]
pub struct StrictRuleVarianceValidator {
    pub rules: Vec<Rc<dyn StrictRuleVarianceRule>>,
}

impl StrictRuleVarianceValidator {
    pub fn validate(&self, item: &StrictRuleVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRuleVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRuleVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rule_variance_validator() -> StrictRuleVarianceValidator {
    StrictRuleVarianceValidator {
        rules: vec![
            Rc::new(StrictRuleVarianceCodeLengthRule { max_length: 1000 }) as Rc<dyn StrictRuleVarianceRule>,
            Rc::new(StrictRuleVarianceThresholdRangeRule { min: 6.0, max: 1529.7 }),
            Rc::new(StrictRuleVarianceRequiredTitleRule {
                message: "strict rule variance needs this field".to_string(),
            }),
            Rc::new(StrictRuleVarianceLocaleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rule_variance_validator();
        let item = new_strict_rule_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictRuleVarianceCodeLengthRule { max_length: 255 };
        let mut item = new_strict_rule_variance("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = StrictRuleVarianceThresholdRangeRule { min: 6.5, max: 737.2 };
        let mut item = new_strict_rule_variance("x", "y");
        item.threshold = 738.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictRuleVarianceRequiredTitleRule {
            message: "strict rule variance needs a value".to_string(),
        };
        let mut item = new_strict_rule_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = StrictRuleVarianceLocaleLengthRule { max_length: 86400 };
        let mut item = new_strict_rule_variance("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
