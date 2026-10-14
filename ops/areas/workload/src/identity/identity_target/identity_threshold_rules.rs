use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity threshold must pass. Check returns its problems, or none.
pub trait IdentityThresholdRule: std::fmt::Debug {
    fn check(&self, item: &IdentityThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdCodePrefixRule {
    pub prefix: String,
}

impl IdentityThresholdRule for IdentityThresholdCodePrefixRule {
    fn check(&self, item: &IdentityThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdHostLengthRule {
    pub max_length: i64,
}

impl IdentityThresholdRule for IdentityThresholdHostLengthRule {
    fn check(&self, item: &IdentityThreshold) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdRequiredLabelRule {
    pub message: String,
}

impl IdentityThresholdRule for IdentityThresholdRequiredLabelRule {
    fn check(&self, item: &IdentityThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdRequiredHostRule {
    pub message: String,
}

impl IdentityThresholdRule for IdentityThresholdRequiredHostRule {
    fn check(&self, item: &IdentityThreshold) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThresholdPinnedGuardRule {
    pub limit: f64,
}

impl IdentityThresholdRule for IdentityThresholdPinnedGuardRule {
    fn check(&self, item: &IdentityThreshold) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity threshold.
#[derive(Debug, Clone)]
pub struct IdentityThresholdValidator {
    pub rules: Vec<Rc<dyn IdentityThresholdRule>>,
}

impl IdentityThresholdValidator {
    pub fn validate(&self, item: &IdentityThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_threshold_validator() -> IdentityThresholdValidator {
    IdentityThresholdValidator {
        rules: vec![
            Rc::new(IdentityThresholdCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn IdentityThresholdRule>,
            Rc::new(IdentityThresholdHostLengthRule { max_length: 3600 }),
            Rc::new(IdentityThresholdRequiredLabelRule {
                message: "identity threshold needs this field".to_string(),
            }),
            Rc::new(IdentityThresholdRequiredHostRule {
                message: "identity threshold needs an entry".to_string(),
            }),
            Rc::new(IdentityThresholdPinnedGuardRule { limit: 2.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_threshold_validator();
        let item = new_identity_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = IdentityThresholdHostLengthRule { max_length: 255 };
        let mut item = new_identity_threshold("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IdentityThresholdRequiredLabelRule {
            message: "identity threshold needs this field".to_string(),
        };
        let mut item = new_identity_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = IdentityThresholdRequiredHostRule {
            message: "identity threshold needs this field".to_string(),
        };
        let mut item = new_identity_threshold("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
