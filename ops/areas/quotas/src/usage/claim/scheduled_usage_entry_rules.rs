use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled usage entry must pass. Check returns its problems, or none.
pub trait ScheduledUsageEntryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledUsageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEntryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledUsageEntryRule for ScheduledUsageEntryCeilingRangeRule {
    fn check(&self, item: &ScheduledUsageEntry) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEntryLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledUsageEntryRule for ScheduledUsageEntryLabelLengthRule {
    fn check(&self, item: &ScheduledUsageEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEntryTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ScheduledUsageEntryRule for ScheduledUsageEntryTagsLimitRule {
    fn check(&self, item: &ScheduledUsageEntry) -> Vec<String> {
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
pub struct ScheduledUsageEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledUsageEntryRule for ScheduledUsageEntryAllowedCodeRule {
    fn check(&self, item: &ScheduledUsageEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled usage entry.
#[derive(Debug, Clone)]
pub struct ScheduledUsageEntryValidator {
    pub rules: Vec<Rc<dyn ScheduledUsageEntryRule>>,
}

impl ScheduledUsageEntryValidator {
    pub fn validate(&self, item: &ScheduledUsageEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledUsageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledUsageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_usage_entry_validator() -> ScheduledUsageEntryValidator {
    ScheduledUsageEntryValidator {
        rules: vec![
            Rc::new(ScheduledUsageEntryCeilingRangeRule { min: 8.7, max: 3487.0 }) as Rc<dyn ScheduledUsageEntryRule>,
            Rc::new(ScheduledUsageEntryLabelLengthRule { max_length: 32 }),
            Rc::new(ScheduledUsageEntryTagsLimitRule {
                max_count: 64,
                banned: vec!["seasonal".to_string(), "capped".to_string()],
            }),
            Rc::new(ScheduledUsageEntryAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "tiered".to_string(), "batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_usage_entry_validator();
        let item = new_scheduled_usage_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ScheduledUsageEntryCeilingRangeRule { min: 9.7, max: 3859.8 };
        let mut item = new_scheduled_usage_entry("x", "y");
        item.ceiling = 3861.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledUsageEntryLabelLengthRule { max_length: 120 };
        let mut item = new_scheduled_usage_entry("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
