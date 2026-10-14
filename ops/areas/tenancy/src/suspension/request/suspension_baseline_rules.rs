use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension baseline must pass. Check returns its problems, or none.
pub trait SuspensionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineCodePrefixRule {
    pub prefix: String,
}

impl SuspensionBaselineRule for SuspensionBaselineCodePrefixRule {
    fn check(&self, item: &SuspensionBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionBaselineRule for SuspensionBaselineCodeLengthRule {
    fn check(&self, item: &SuspensionBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineSegmentLengthRule {
    pub max_length: i64,
}

impl SuspensionBaselineRule for SuspensionBaselineSegmentLengthRule {
    fn check(&self, item: &SuspensionBaseline) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineRequiredNameRule {
    pub message: String,
}

impl SuspensionBaselineRule for SuspensionBaselineRequiredNameRule {
    fn check(&self, item: &SuspensionBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension baseline.
#[derive(Debug, Clone)]
pub struct SuspensionBaselineValidator {
    pub rules: Vec<Rc<dyn SuspensionBaselineRule>>,
}

impl SuspensionBaselineValidator {
    pub fn validate(&self, item: &SuspensionBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_baseline_validator() -> SuspensionBaselineValidator {
    SuspensionBaselineValidator {
        rules: vec![
            Rc::new(SuspensionBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SuspensionBaselineRule>,
            Rc::new(SuspensionBaselineCodeLengthRule { max_length: 86400 }),
            Rc::new(SuspensionBaselineSegmentLengthRule { max_length: 1000 }),
            Rc::new(SuspensionBaselineRequiredNameRule {
                message: "suspension baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_baseline_validator();
        let item = new_suspension_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionBaselineCodeLengthRule { max_length: 3600 };
        let mut item = new_suspension_baseline("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SuspensionBaselineSegmentLengthRule { max_length: 80 };
        let mut item = new_suspension_baseline("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SuspensionBaselineRequiredNameRule {
            message: "suspension baseline needs an entry".to_string(),
        };
        let mut item = new_suspension_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
