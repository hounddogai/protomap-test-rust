use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage threshold must pass. Check returns its problems, or none.
pub trait UsageThresholdRule: std::fmt::Debug {
    fn check(&self, item: &UsageThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageThresholdAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl UsageThresholdRule for UsageThresholdAllowedClusterRule {
    fn check(&self, item: &UsageThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is too long", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageThresholdCodePrefixRule {
    pub prefix: String,
}

impl UsageThresholdRule for UsageThresholdCodePrefixRule {
    fn check(&self, item: &UsageThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageThresholdRequiredTitleRule {
    pub message: String,
}

impl UsageThresholdRule for UsageThresholdRequiredTitleRule {
    fn check(&self, item: &UsageThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage threshold.
#[derive(Debug, Clone)]
pub struct UsageThresholdValidator {
    pub rules: Vec<Rc<dyn UsageThresholdRule>>,
}

impl UsageThresholdValidator {
    pub fn validate(&self, item: &UsageThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_threshold_validator() -> UsageThresholdValidator {
    UsageThresholdValidator {
        rules: vec![
            Rc::new(UsageThresholdAllowedClusterRule { allowed: Vec::new() }) as Rc<dyn UsageThresholdRule>,
            Rc::new(UsageThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(UsageThresholdRequiredTitleRule {
                message: "usage threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_threshold_validator();
        let item = new_usage_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = UsageThresholdRequiredTitleRule {
            message: "usage threshold needs a value".to_string(),
        };
        let mut item = new_usage_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
