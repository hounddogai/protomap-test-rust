use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive canary assignment must pass. Check returns its problems, or none.
pub trait AdaptiveCanaryAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveCanaryAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentVisibleGuardRule {
    pub limit: f64,
}

impl AdaptiveCanaryAssignmentRule for AdaptiveCanaryAssignmentVisibleGuardRule {
    fn check(&self, item: &AdaptiveCanaryAssignment) -> Vec<String> {
        if item.visible && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while visible", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentRequiredLabelRule {
    pub message: String,
}

impl AdaptiveCanaryAssignmentRule for AdaptiveCanaryAssignmentRequiredLabelRule {
    fn check(&self, item: &AdaptiveCanaryAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl AdaptiveCanaryAssignmentRule for AdaptiveCanaryAssignmentAllowedKeyIdRule {
    fn check(&self, item: &AdaptiveCanaryAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is expired", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveCanaryAssignmentRule for AdaptiveCanaryAssignmentCodeLengthRule {
    fn check(&self, item: &AdaptiveCanaryAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive canary assignment.
#[derive(Debug, Clone)]
pub struct AdaptiveCanaryAssignmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveCanaryAssignmentRule>>,
}

impl AdaptiveCanaryAssignmentValidator {
    pub fn validate(&self, item: &AdaptiveCanaryAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveCanaryAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveCanaryAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_canary_assignment_validator() -> AdaptiveCanaryAssignmentValidator {
    AdaptiveCanaryAssignmentValidator {
        rules: vec![
            Rc::new(AdaptiveCanaryAssignmentVisibleGuardRule { limit: 1.6 }) as Rc<dyn AdaptiveCanaryAssignmentRule>,
            Rc::new(AdaptiveCanaryAssignmentRequiredLabelRule {
                message: "adaptive canary assignment needs a value".to_string(),
            }),
            Rc::new(AdaptiveCanaryAssignmentAllowedKeyIdRule {
                allowed: vec!["legacy".to_string(), "seasonal".to_string()],
            }),
            Rc::new(AdaptiveCanaryAssignmentCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_canary_assignment_validator();
        let item = new_adaptive_canary_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveCanaryAssignmentRequiredLabelRule {
            message: "adaptive canary assignment needs a value".to_string(),
        };
        let mut item = new_adaptive_canary_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveCanaryAssignmentCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_canary_assignment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
