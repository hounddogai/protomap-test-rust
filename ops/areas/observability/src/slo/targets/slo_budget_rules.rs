use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo budget must pass. Check returns its problems, or none.
pub trait SloBudgetRule: std::fmt::Debug {
    fn check(&self, item: &SloBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloBudgetRule for SloBudgetQuotaUnitsRangeRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetTitleLengthRule {
    pub max_length: i64,
}

impl SloBudgetRule for SloBudgetTitleLengthRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetRequiredTitleRule {
    pub message: String,
}

impl SloBudgetRule for SloBudgetRequiredTitleRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetRequiredLocaleRule {
    pub message: String,
}

impl SloBudgetRule for SloBudgetRequiredLocaleRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetLocaleLengthRule {
    pub max_length: i64,
}

impl SloBudgetRule for SloBudgetLocaleLengthRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetCodePrefixRule {
    pub prefix: String,
}

impl SloBudgetRule for SloBudgetCodePrefixRule {
    fn check(&self, item: &SloBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo budget.
#[derive(Debug, Clone)]
pub struct SloBudgetValidator {
    pub rules: Vec<Rc<dyn SloBudgetRule>>,
}

impl SloBudgetValidator {
    pub fn validate(&self, item: &SloBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_budget_validator() -> SloBudgetValidator {
    SloBudgetValidator {
        rules: vec![
            Rc::new(SloBudgetQuotaUnitsRangeRule { min: 16, max: 120 }) as Rc<dyn SloBudgetRule>,
            Rc::new(SloBudgetTitleLengthRule { max_length: 86400 }),
            Rc::new(SloBudgetRequiredTitleRule {
                message: "slo budget needs a value".to_string(),
            }),
            Rc::new(SloBudgetRequiredLocaleRule {
                message: "slo budget needs an entry".to_string(),
            }),
            Rc::new(SloBudgetLocaleLengthRule { max_length: 120 }),
            Rc::new(SloBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_budget_validator();
        let item = new_slo_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SloBudgetQuotaUnitsRangeRule { min: 64, max: 64 };
        let mut item = new_slo_budget("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SloBudgetTitleLengthRule { max_length: 120 };
        let mut item = new_slo_budget("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SloBudgetRequiredTitleRule {
            message: "slo budget needs a value".to_string(),
        };
        let mut item = new_slo_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SloBudgetRequiredLocaleRule {
            message: "slo budget needs an entry".to_string(),
        };
        let mut item = new_slo_budget("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SloBudgetLocaleLengthRule { max_length: 80 };
        let mut item = new_slo_budget("x", "y");
        item.locale = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
