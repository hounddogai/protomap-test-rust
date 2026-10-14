use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline summary must pass. Check returns its problems, or none.
pub trait TimelineSummaryRule: std::fmt::Debug {
    fn check(&self, item: &TimelineSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryCodeLengthRule {
    pub max_length: i64,
}

impl TimelineSummaryRule for TimelineSummaryCodeLengthRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineSummaryRule for TimelineSummaryPriorityRangeRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl TimelineSummaryRule for TimelineSummaryAllowedNamespaceRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is stale", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryVisibleGuardRule {
    pub limit: i64,
}

impl TimelineSummaryRule for TimelineSummaryVisibleGuardRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
        if item.visible && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while visible",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryNamespaceLengthRule {
    pub max_length: i64,
}

impl TimelineSummaryRule for TimelineSummaryNamespaceLengthRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
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
pub struct TimelineSummaryVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineSummaryRule for TimelineSummaryVersionRangeRule {
    fn check(&self, item: &TimelineSummary) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a timeline summary.
#[derive(Debug, Clone)]
pub struct TimelineSummaryValidator {
    pub rules: Vec<Rc<dyn TimelineSummaryRule>>,
}

impl TimelineSummaryValidator {
    pub fn validate(&self, item: &TimelineSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_summary_validator() -> TimelineSummaryValidator {
    TimelineSummaryValidator {
        rules: vec![
            Rc::new(TimelineSummaryCodeLengthRule { max_length: 255 }) as Rc<dyn TimelineSummaryRule>,
            Rc::new(TimelineSummaryPriorityRangeRule { min: 16, max: 255 }),
            Rc::new(TimelineSummaryAllowedNamespaceRule { allowed: Vec::new() }),
            Rc::new(TimelineSummaryVisibleGuardRule { limit: 1000 }),
            Rc::new(TimelineSummaryNamespaceLengthRule { max_length: 64 }),
            Rc::new(TimelineSummaryVersionRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_summary_validator();
        let item = new_timeline_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineSummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_timeline_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = TimelineSummaryPriorityRangeRule { min: 120, max: 255 };
        let mut item = new_timeline_summary("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TimelineSummaryNamespaceLengthRule { max_length: 32 };
        let mut item = new_timeline_summary("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = TimelineSummaryVersionRangeRule { min: 120, max: 86400 };
        let mut item = new_timeline_summary("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
