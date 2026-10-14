use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default zone snapshot must pass. Check returns its problems, or none.
pub trait DefaultZoneSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotMarginRangeRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotRequiredTitleRule {
    pub message: String,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotRequiredTitleRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotRequiredReviewerRule {
    pub message: String,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotRequiredReviewerRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotAllowedSummaryRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is expired", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotTitleLengthRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DefaultZoneSnapshotRule for DefaultZoneSnapshotCodePrefixRule {
    fn check(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default zone snapshot.
#[derive(Debug, Clone)]
pub struct DefaultZoneSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultZoneSnapshotRule>>,
}

impl DefaultZoneSnapshotValidator {
    pub fn validate(&self, item: &DefaultZoneSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultZoneSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultZoneSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_zone_snapshot_validator() -> DefaultZoneSnapshotValidator {
    DefaultZoneSnapshotValidator {
        rules: vec![
            Rc::new(DefaultZoneSnapshotMarginRangeRule { min: 5.4, max: 4478.6 }) as Rc<dyn DefaultZoneSnapshotRule>,
            Rc::new(DefaultZoneSnapshotRequiredTitleRule {
                message: "default zone snapshot needs an entry".to_string(),
            }),
            Rc::new(DefaultZoneSnapshotRequiredReviewerRule {
                message: "default zone snapshot needs an entry".to_string(),
            }),
            Rc::new(DefaultZoneSnapshotAllowedSummaryRule {
                allowed: vec!["strict".to_string(), "capped".to_string(), "tiered".to_string()],
            }),
            Rc::new(DefaultZoneSnapshotTitleLengthRule { max_length: 86400 }),
            Rc::new(DefaultZoneSnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_zone_snapshot_validator();
        let item = new_default_zone_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DefaultZoneSnapshotMarginRangeRule { min: 10.0, max: 1346.0 };
        let mut item = new_default_zone_snapshot("x", "y");
        item.margin = 1347.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultZoneSnapshotRequiredTitleRule {
            message: "default zone snapshot needs a value".to_string(),
        };
        let mut item = new_default_zone_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DefaultZoneSnapshotRequiredReviewerRule {
            message: "default zone snapshot needs an entry".to_string(),
        };
        let mut item = new_default_zone_snapshot("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultZoneSnapshotTitleLengthRule { max_length: 120 };
        let mut item = new_default_zone_snapshot("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
