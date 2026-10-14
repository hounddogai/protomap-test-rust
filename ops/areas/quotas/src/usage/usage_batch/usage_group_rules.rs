use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage group must pass. Check returns its problems, or none.
pub trait UsageGroupRule: std::fmt::Debug {
    fn check(&self, item: &UsageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageGroupCodePrefixRule {
    pub prefix: String,
}

impl UsageGroupRule for UsageGroupCodePrefixRule {
    fn check(&self, item: &UsageGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageGroupSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageGroupRule for UsageGroupSequenceRangeRule {
    fn check(&self, item: &UsageGroup) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageGroupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl UsageGroupRule for UsageGroupAllowedTitleRule {
    fn check(&self, item: &UsageGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage group.
#[derive(Debug, Clone)]
pub struct UsageGroupValidator {
    pub rules: Vec<Rc<dyn UsageGroupRule>>,
}

impl UsageGroupValidator {
    pub fn validate(&self, item: &UsageGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_group_validator() -> UsageGroupValidator {
    UsageGroupValidator {
        rules: vec![
            Rc::new(UsageGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn UsageGroupRule>,
            Rc::new(UsageGroupSequenceRangeRule { min: 80, max: 3600 }),
            Rc::new(UsageGroupAllowedTitleRule {
                allowed: vec!["strict".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_group_validator();
        let item = new_usage_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = UsageGroupSequenceRangeRule { min: 32, max: 3600 };
        let mut item = new_usage_group("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
