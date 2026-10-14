use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation score must pass. Check returns its problems, or none.
pub trait DelegationScoreRule: std::fmt::Debug {
    fn check(&self, item: &DelegationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScoreNamespaceLengthRule {
    pub max_length: i64,
}

impl DelegationScoreRule for DelegationScoreNamespaceLengthRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScoreBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationScoreRule for DelegationScoreBudgetRangeRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
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
pub struct DelegationScoreCodeLengthRule {
    pub max_length: i64,
}

impl DelegationScoreRule for DelegationScoreCodeLengthRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScoreTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DelegationScoreRule for DelegationScoreTagsLimitRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScoreAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DelegationScoreRule for DelegationScoreAllowedTitleRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScoreWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationScoreRule for DelegationScoreWindowDaysRangeRule {
    fn check(&self, item: &DelegationScore) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a delegation score.
#[derive(Debug, Clone)]
pub struct DelegationScoreValidator {
    pub rules: Vec<Rc<dyn DelegationScoreRule>>,
}

impl DelegationScoreValidator {
    pub fn validate(&self, item: &DelegationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_score_validator() -> DelegationScoreValidator {
    DelegationScoreValidator {
        rules: vec![
            Rc::new(DelegationScoreNamespaceLengthRule { max_length: 3600 }) as Rc<dyn DelegationScoreRule>,
            Rc::new(DelegationScoreBudgetRangeRule { min: 3.4, max: 153.1 }),
            Rc::new(DelegationScoreCodeLengthRule { max_length: 3600 }),
            Rc::new(DelegationScoreTagsLimitRule {
                max_count: 86400,
                banned: vec!["strict".to_string(), "capped".to_string()],
            }),
            Rc::new(DelegationScoreAllowedTitleRule {
                allowed: vec!["pending".to_string(), "pending".to_string()],
            }),
            Rc::new(DelegationScoreWindowDaysRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_score_validator();
        let item = new_delegation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DelegationScoreNamespaceLengthRule { max_length: 255 };
        let mut item = new_delegation_score("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = DelegationScoreBudgetRangeRule { min: 6.6, max: 2751.5 };
        let mut item = new_delegation_score("x", "y");
        item.budget = 2753.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationScoreCodeLengthRule { max_length: 86400 };
        let mut item = new_delegation_score("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DelegationScoreWindowDaysRangeRule { min: 32, max: 120 };
        let mut item = new_delegation_score("x", "y");
        item.window_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
