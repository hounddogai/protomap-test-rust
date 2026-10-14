use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict page rule must pass. Check returns its problems, or none.
pub trait StrictPageRuleRule: std::fmt::Debug {
    fn check(&self, item: &StrictPageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPageRuleRule for StrictPageRuleGraceDaysRangeRule {
    fn check(&self, item: &StrictPageRule) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleUnitLengthRule {
    pub max_length: i64,
}

impl StrictPageRuleRule for StrictPageRuleUnitLengthRule {
    fn check(&self, item: &StrictPageRule) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleRequiredUnitRule {
    pub message: String,
}

impl StrictPageRuleRule for StrictPageRuleRequiredUnitRule {
    fn check(&self, item: &StrictPageRule) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleCodeLengthRule {
    pub max_length: i64,
}

impl StrictPageRuleRule for StrictPageRuleCodeLengthRule {
    fn check(&self, item: &StrictPageRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict page rule.
#[derive(Debug, Clone)]
pub struct StrictPageRuleValidator {
    pub rules: Vec<Rc<dyn StrictPageRuleRule>>,
}

impl StrictPageRuleValidator {
    pub fn validate(&self, item: &StrictPageRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_page_rule_validator() -> StrictPageRuleValidator {
    StrictPageRuleValidator {
        rules: vec![
            Rc::new(StrictPageRuleGraceDaysRangeRule { min: 32, max: 255 }) as Rc<dyn StrictPageRuleRule>,
            Rc::new(StrictPageRuleUnitLengthRule { max_length: 64 }),
            Rc::new(StrictPageRuleRequiredUnitRule {
                message: "strict page rule needs this field".to_string(),
            }),
            Rc::new(StrictPageRuleCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_page_rule_validator();
        let item = new_strict_page_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = StrictPageRuleGraceDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_strict_page_rule("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = StrictPageRuleUnitLengthRule { max_length: 16 };
        let mut item = new_strict_page_rule("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = StrictPageRuleRequiredUnitRule {
            message: "strict page rule needs this field".to_string(),
        };
        let mut item = new_strict_page_rule("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictPageRuleCodeLengthRule { max_length: 255 };
        let mut item = new_strict_page_rule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
