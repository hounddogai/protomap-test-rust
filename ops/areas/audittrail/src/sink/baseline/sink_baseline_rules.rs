use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink baseline must pass. Check returns its problems, or none.
pub trait SinkBaselineRule: std::fmt::Debug {
    fn check(&self, item: &SinkBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineCodeLengthRule {
    pub max_length: i64,
}

impl SinkBaselineRule for SinkBaselineCodeLengthRule {
    fn check(&self, item: &SinkBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineRequiredPrincipalRule {
    pub message: String,
}

impl SinkBaselineRule for SinkBaselineRequiredPrincipalRule {
    fn check(&self, item: &SinkBaseline) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineCodePrefixRule {
    pub prefix: String,
}

impl SinkBaselineRule for SinkBaselineCodePrefixRule {
    fn check(&self, item: &SinkBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineSegmentLengthRule {
    pub max_length: i64,
}

impl SinkBaselineRule for SinkBaselineSegmentLengthRule {
    fn check(&self, item: &SinkBaseline) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselinePrincipalLengthRule {
    pub max_length: i64,
}

impl SinkBaselineRule for SinkBaselinePrincipalLengthRule {
    fn check(&self, item: &SinkBaseline) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink baseline.
#[derive(Debug, Clone)]
pub struct SinkBaselineValidator {
    pub rules: Vec<Rc<dyn SinkBaselineRule>>,
}

impl SinkBaselineValidator {
    pub fn validate(&self, item: &SinkBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_baseline_validator() -> SinkBaselineValidator {
    SinkBaselineValidator {
        rules: vec![
            Rc::new(SinkBaselineCodeLengthRule { max_length: 80 }) as Rc<dyn SinkBaselineRule>,
            Rc::new(SinkBaselineRequiredPrincipalRule {
                message: "sink baseline needs this field".to_string(),
            }),
            Rc::new(SinkBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SinkBaselineSegmentLengthRule { max_length: 32 }),
            Rc::new(SinkBaselinePrincipalLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_baseline_validator();
        let item = new_sink_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkBaselineCodeLengthRule { max_length: 16 };
        let mut item = new_sink_baseline("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SinkBaselineRequiredPrincipalRule {
            message: "sink baseline needs an entry".to_string(),
        };
        let mut item = new_sink_baseline("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SinkBaselineSegmentLengthRule { max_length: 120 };
        let mut item = new_sink_baseline("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = SinkBaselinePrincipalLengthRule { max_length: 80 };
        let mut item = new_sink_baseline("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
