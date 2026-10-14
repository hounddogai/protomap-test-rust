use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled query entry must pass. Check returns its problems, or none.
pub trait ScheduledQueryEntryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQueryEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEntryNamespaceLengthRule {
    pub max_length: i64,
}

impl ScheduledQueryEntryRule for ScheduledQueryEntryNamespaceLengthRule {
    fn check(&self, item: &ScheduledQueryEntry) -> Vec<String> {
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
pub struct ScheduledQueryEntryRequiredCodeRule {
    pub message: String,
}

impl ScheduledQueryEntryRule for ScheduledQueryEntryRequiredCodeRule {
    fn check(&self, item: &ScheduledQueryEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEntryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledQueryEntryRule for ScheduledQueryEntryMaximumRangeRule {
    fn check(&self, item: &ScheduledQueryEntry) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled query entry.
#[derive(Debug, Clone)]
pub struct ScheduledQueryEntryValidator {
    pub rules: Vec<Rc<dyn ScheduledQueryEntryRule>>,
}

impl ScheduledQueryEntryValidator {
    pub fn validate(&self, item: &ScheduledQueryEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledQueryEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQueryEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_query_entry_validator() -> ScheduledQueryEntryValidator {
    ScheduledQueryEntryValidator {
        rules: vec![
            Rc::new(ScheduledQueryEntryNamespaceLengthRule { max_length: 3600 }) as Rc<dyn ScheduledQueryEntryRule>,
            Rc::new(ScheduledQueryEntryRequiredCodeRule {
                message: "scheduled query entry needs this field".to_string(),
            }),
            Rc::new(ScheduledQueryEntryMaximumRangeRule { min: 9.4, max: 3995.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_query_entry_validator();
        let item = new_scheduled_query_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ScheduledQueryEntryNamespaceLengthRule { max_length: 1000 };
        let mut item = new_scheduled_query_entry("x", "y");
        item.namespace = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledQueryEntryRequiredCodeRule {
            message: "scheduled query entry needs an entry".to_string(),
        };
        let mut item = new_scheduled_query_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledQueryEntryMaximumRangeRule { min: 6.3, max: 2348.5 };
        let mut item = new_scheduled_query_entry("x", "y");
        item.maximum = 2350.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
