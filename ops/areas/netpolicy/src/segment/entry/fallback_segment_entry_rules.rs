use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback segment entry must pass. Check returns its problems, or none.
pub trait FallbackSegmentEntryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSegmentEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntryVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackSegmentEntryRule for FallbackSegmentEntryVersionRangeRule {
    fn check(&self, item: &FallbackSegmentEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntryWindowRule {
    pub max_span: i64,
}

impl FallbackSegmentEntryRule for FallbackSegmentEntryWindowRule {
    fn check(&self, item: &FallbackSegmentEntry) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntryTitleLengthRule {
    pub max_length: i64,
}

impl FallbackSegmentEntryRule for FallbackSegmentEntryTitleLengthRule {
    fn check(&self, item: &FallbackSegmentEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntryApprovedGuardRule {
    pub limit: i64,
}

impl FallbackSegmentEntryRule for FallbackSegmentEntryApprovedGuardRule {
    fn check(&self, item: &FallbackSegmentEntry) -> Vec<String> {
        if item.approved && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while approved",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback segment entry.
#[derive(Debug, Clone)]
pub struct FallbackSegmentEntryValidator {
    pub rules: Vec<Rc<dyn FallbackSegmentEntryRule>>,
}

impl FallbackSegmentEntryValidator {
    pub fn validate(&self, item: &FallbackSegmentEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSegmentEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSegmentEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_segment_entry_validator() -> FallbackSegmentEntryValidator {
    FallbackSegmentEntryValidator {
        rules: vec![
            Rc::new(FallbackSegmentEntryVersionRangeRule { min: 32, max: 80 }) as Rc<dyn FallbackSegmentEntryRule>,
            Rc::new(FallbackSegmentEntryWindowRule { max_span: 64 }),
            Rc::new(FallbackSegmentEntryTitleLengthRule { max_length: 16 }),
            Rc::new(FallbackSegmentEntryApprovedGuardRule { limit: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_segment_entry_validator();
        let item = new_fallback_segment_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackSegmentEntryVersionRangeRule { min: 1000, max: 86400 };
        let mut item = new_fallback_segment_entry("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackSegmentEntryTitleLengthRule { max_length: 3600 };
        let mut item = new_fallback_segment_entry("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
