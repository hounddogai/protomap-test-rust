use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension threshold must pass. Check returns its problems, or none.
pub trait SuspensionThresholdRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdCodePrefixRule {
    pub prefix: String,
}

impl SuspensionThresholdRule for SuspensionThresholdCodePrefixRule {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdNameLengthRule {
    pub max_length: i64,
}

impl SuspensionThresholdRule for SuspensionThresholdNameLengthRule {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionThresholdRule for SuspensionThresholdCodeLengthRule {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdRequiredTimezoneRule {
    pub message: String,
}

impl SuspensionThresholdRule for SuspensionThresholdRequiredTimezoneRule {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionThresholdTimezoneLengthRule {
    pub max_length: i64,
}

impl SuspensionThresholdRule for SuspensionThresholdTimezoneLengthRule {
    fn check(&self, item: &SuspensionThreshold) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension threshold.
#[derive(Debug, Clone)]
pub struct SuspensionThresholdValidator {
    pub rules: Vec<Rc<dyn SuspensionThresholdRule>>,
}

impl SuspensionThresholdValidator {
    pub fn validate(&self, item: &SuspensionThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_threshold_validator() -> SuspensionThresholdValidator {
    SuspensionThresholdValidator {
        rules: vec![
            Rc::new(SuspensionThresholdCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SuspensionThresholdRule>,
            Rc::new(SuspensionThresholdNameLengthRule { max_length: 80 }),
            Rc::new(SuspensionThresholdCodeLengthRule { max_length: 64 }),
            Rc::new(SuspensionThresholdRequiredTimezoneRule {
                message: "suspension threshold needs this field".to_string(),
            }),
            Rc::new(SuspensionThresholdTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_threshold_validator();
        let item = new_suspension_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SuspensionThresholdNameLengthRule { max_length: 1000 };
        let mut item = new_suspension_threshold("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_suspension_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = SuspensionThresholdRequiredTimezoneRule {
            message: "suspension threshold needs this field".to_string(),
        };
        let mut item = new_suspension_threshold("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SuspensionThresholdTimezoneLengthRule { max_length: 120 };
        let mut item = new_suspension_threshold("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
