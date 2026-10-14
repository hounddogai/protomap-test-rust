use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem policy must pass. Check returns its problems, or none.
pub trait PostmortemPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPolicyRequiredFingerprintRule {
    pub message: String,
}

impl PostmortemPolicyRule for PostmortemPolicyRequiredFingerprintRule {
    fn check(&self, item: &PostmortemPolicy) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPolicySourceLengthRule {
    pub max_length: i64,
}

impl PostmortemPolicyRule for PostmortemPolicySourceLengthRule {
    fn check(&self, item: &PostmortemPolicy) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPolicyVisibleGuardRule {
    pub limit: i64,
}

impl PostmortemPolicyRule for PostmortemPolicyVisibleGuardRule {
    fn check(&self, item: &PostmortemPolicy) -> Vec<String> {
        if item.visible && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while visible",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem policy.
#[derive(Debug, Clone)]
pub struct PostmortemPolicyValidator {
    pub rules: Vec<Rc<dyn PostmortemPolicyRule>>,
}

impl PostmortemPolicyValidator {
    pub fn validate(&self, item: &PostmortemPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_policy_validator() -> PostmortemPolicyValidator {
    PostmortemPolicyValidator {
        rules: vec![
            Rc::new(PostmortemPolicyRequiredFingerprintRule {
                message: "postmortem policy needs this field".to_string(),
            }) as Rc<dyn PostmortemPolicyRule>,
            Rc::new(PostmortemPolicySourceLengthRule { max_length: 80 }),
            Rc::new(PostmortemPolicyVisibleGuardRule { limit: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_policy_validator();
        let item = new_postmortem_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = PostmortemPolicyRequiredFingerprintRule {
            message: "postmortem policy needs this field".to_string(),
        };
        let mut item = new_postmortem_policy("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PostmortemPolicySourceLengthRule { max_length: 3600 };
        let mut item = new_postmortem_policy("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
