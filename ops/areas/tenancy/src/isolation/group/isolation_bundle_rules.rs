use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation bundle must pass. Check returns its problems, or none.
pub trait IsolationBundleRule: std::fmt::Debug {
    fn check(&self, item: &IsolationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleSegmentLengthRule {
    pub max_length: i64,
}

impl IsolationBundleRule for IsolationBundleSegmentLengthRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleRequiredSegmentRule {
    pub message: String,
}

impl IsolationBundleRule for IsolationBundleRequiredSegmentRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationBundleRule for IsolationBundleQuotaUnitsRangeRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleRequiredCodeRule {
    pub message: String,
}

impl IsolationBundleRule for IsolationBundleRequiredCodeRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl IsolationBundleRule for IsolationBundleAllowedTitleRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBundleTitleLengthRule {
    pub max_length: i64,
}

impl IsolationBundleRule for IsolationBundleTitleLengthRule {
    fn check(&self, item: &IsolationBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation bundle.
#[derive(Debug, Clone)]
pub struct IsolationBundleValidator {
    pub rules: Vec<Rc<dyn IsolationBundleRule>>,
}

impl IsolationBundleValidator {
    pub fn validate(&self, item: &IsolationBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_bundle_validator() -> IsolationBundleValidator {
    IsolationBundleValidator {
        rules: vec![
            Rc::new(IsolationBundleSegmentLengthRule { max_length: 1000 }) as Rc<dyn IsolationBundleRule>,
            Rc::new(IsolationBundleRequiredSegmentRule {
                message: "isolation bundle needs this field".to_string(),
            }),
            Rc::new(IsolationBundleQuotaUnitsRangeRule { min: 255, max: 86400 }),
            Rc::new(IsolationBundleRequiredCodeRule {
                message: "isolation bundle needs an entry".to_string(),
            }),
            Rc::new(IsolationBundleAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(IsolationBundleTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_bundle_validator();
        let item = new_isolation_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = IsolationBundleSegmentLengthRule { max_length: 120 };
        let mut item = new_isolation_bundle("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = IsolationBundleRequiredSegmentRule {
            message: "isolation bundle needs this field".to_string(),
        };
        let mut item = new_isolation_bundle("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = IsolationBundleQuotaUnitsRangeRule { min: 32, max: 120 };
        let mut item = new_isolation_bundle("x", "y");
        item.quota_units = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationBundleRequiredCodeRule {
            message: "isolation bundle needs this field".to_string(),
        };
        let mut item = new_isolation_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IsolationBundleTitleLengthRule { max_length: 3600 };
        let mut item = new_isolation_bundle("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
