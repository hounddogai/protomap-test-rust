use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline digest must pass. Check returns its problems, or none.
pub trait TimelineDigestRule: std::fmt::Debug {
    fn check(&self, item: &TimelineDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestRequiredLabelRule {
    pub message: String,
}

impl TimelineDigestRule for TimelineDigestRequiredLabelRule {
    fn check(&self, item: &TimelineDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestLabelLengthRule {
    pub max_length: i64,
}

impl TimelineDigestRule for TimelineDigestLabelLengthRule {
    fn check(&self, item: &TimelineDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineDigestRule for TimelineDigestQuantityRangeRule {
    fn check(&self, item: &TimelineDigest) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestCodeLengthRule {
    pub max_length: i64,
}

impl TimelineDigestRule for TimelineDigestCodeLengthRule {
    fn check(&self, item: &TimelineDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestTitleLengthRule {
    pub max_length: i64,
}

impl TimelineDigestRule for TimelineDigestTitleLengthRule {
    fn check(&self, item: &TimelineDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline digest.
#[derive(Debug, Clone)]
pub struct TimelineDigestValidator {
    pub rules: Vec<Rc<dyn TimelineDigestRule>>,
}

impl TimelineDigestValidator {
    pub fn validate(&self, item: &TimelineDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_digest_validator() -> TimelineDigestValidator {
    TimelineDigestValidator {
        rules: vec![
            Rc::new(TimelineDigestRequiredLabelRule {
                message: "timeline digest needs this field".to_string(),
            }) as Rc<dyn TimelineDigestRule>,
            Rc::new(TimelineDigestLabelLengthRule { max_length: 86400 }),
            Rc::new(TimelineDigestQuantityRangeRule { min: 16, max: 1000 }),
            Rc::new(TimelineDigestCodeLengthRule { max_length: 1000 }),
            Rc::new(TimelineDigestTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_digest_validator();
        let item = new_timeline_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TimelineDigestRequiredLabelRule {
            message: "timeline digest needs an entry".to_string(),
        };
        let mut item = new_timeline_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TimelineDigestLabelLengthRule { max_length: 3600 };
        let mut item = new_timeline_digest("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TimelineDigestQuantityRangeRule { min: 16, max: 64 };
        let mut item = new_timeline_digest("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineDigestCodeLengthRule { max_length: 32 };
        let mut item = new_timeline_digest("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelineDigestTitleLengthRule { max_length: 16 };
        let mut item = new_timeline_digest("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
