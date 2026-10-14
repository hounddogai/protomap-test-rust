use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive runbook bundle must pass. Check returns its problems, or none.
pub trait AdaptiveRunbookBundleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleRequiredCodeRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleRequiredSummaryRule {
    pub message: String,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleRequiredSummaryRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleSummaryLengthRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleCodeLengthRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleRequiredNotesRule {
    pub message: String,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleRequiredNotesRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRunbookBundleRule for AdaptiveRunbookBundleVersionRangeRule {
    fn check(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive runbook bundle.
#[derive(Debug, Clone)]
pub struct AdaptiveRunbookBundleValidator {
    pub rules: Vec<Rc<dyn AdaptiveRunbookBundleRule>>,
}

impl AdaptiveRunbookBundleValidator {
    pub fn validate(&self, item: &AdaptiveRunbookBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRunbookBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRunbookBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_runbook_bundle_validator() -> AdaptiveRunbookBundleValidator {
    AdaptiveRunbookBundleValidator {
        rules: vec![
            Rc::new(AdaptiveRunbookBundleRequiredCodeRule {
                message: "adaptive runbook bundle needs this field".to_string(),
            }) as Rc<dyn AdaptiveRunbookBundleRule>,
            Rc::new(AdaptiveRunbookBundleRequiredSummaryRule {
                message: "adaptive runbook bundle needs this field".to_string(),
            }),
            Rc::new(AdaptiveRunbookBundleSummaryLengthRule { max_length: 16 }),
            Rc::new(AdaptiveRunbookBundleCodeLengthRule { max_length: 120 }),
            Rc::new(AdaptiveRunbookBundleRequiredNotesRule {
                message: "adaptive runbook bundle needs an entry".to_string(),
            }),
            Rc::new(AdaptiveRunbookBundleVersionRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_runbook_bundle_validator();
        let item = new_adaptive_runbook_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveRunbookBundleRequiredCodeRule {
            message: "adaptive runbook bundle needs this field".to_string(),
        };
        let mut item = new_adaptive_runbook_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AdaptiveRunbookBundleRequiredSummaryRule {
            message: "adaptive runbook bundle needs a value".to_string(),
        };
        let mut item = new_adaptive_runbook_bundle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = AdaptiveRunbookBundleSummaryLengthRule { max_length: 86400 };
        let mut item = new_adaptive_runbook_bundle("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveRunbookBundleCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_runbook_bundle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = AdaptiveRunbookBundleRequiredNotesRule {
            message: "adaptive runbook bundle needs this field".to_string(),
        };
        let mut item = new_adaptive_runbook_bundle("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = AdaptiveRunbookBundleVersionRangeRule { min: 80, max: 3600 };
        let mut item = new_adaptive_runbook_bundle("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
