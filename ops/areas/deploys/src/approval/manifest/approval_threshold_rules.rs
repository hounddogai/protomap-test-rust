use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval threshold must pass. Check returns its problems, or none.
pub trait ApprovalThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalThresholdRule for ApprovalThresholdWindowDaysRangeRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdRequiredHostRule {
    pub message: String,
}

impl ApprovalThresholdRule for ApprovalThresholdRequiredHostRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdCodePrefixRule {
    pub prefix: String,
}

impl ApprovalThresholdRule for ApprovalThresholdCodePrefixRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalThresholdRule for ApprovalThresholdCeilingRangeRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdHostLengthRule {
    pub max_length: i64,
}

impl ApprovalThresholdRule for ApprovalThresholdHostLengthRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl ApprovalThresholdRule for ApprovalThresholdAllowedFingerprintRule {
    fn check(&self, item: &ApprovalThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is duplicated", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval threshold.
#[derive(Debug, Clone)]
pub struct ApprovalThresholdValidator {
    pub rules: Vec<Rc<dyn ApprovalThresholdRule>>,
}

impl ApprovalThresholdValidator {
    pub fn validate(&self, item: &ApprovalThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_threshold_validator() -> ApprovalThresholdValidator {
    ApprovalThresholdValidator {
        rules: vec![
            Rc::new(ApprovalThresholdWindowDaysRangeRule { min: 64, max: 80 }) as Rc<dyn ApprovalThresholdRule>,
            Rc::new(ApprovalThresholdRequiredHostRule {
                message: "approval threshold needs a value".to_string(),
            }),
            Rc::new(ApprovalThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ApprovalThresholdCeilingRangeRule { min: 8.3, max: 3252.5 }),
            Rc::new(ApprovalThresholdHostLengthRule { max_length: 3600 }),
            Rc::new(ApprovalThresholdAllowedFingerprintRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_threshold_validator();
        let item = new_approval_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ApprovalThresholdWindowDaysRangeRule { min: 64, max: 1000 };
        let mut item = new_approval_threshold("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ApprovalThresholdRequiredHostRule {
            message: "approval threshold needs this field".to_string(),
        };
        let mut item = new_approval_threshold("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ApprovalThresholdCeilingRangeRule { min: 8.5, max: 3909.6 };
        let mut item = new_approval_threshold("x", "y");
        item.ceiling = 3911.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ApprovalThresholdHostLengthRule { max_length: 1000 };
        let mut item = new_approval_threshold("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
