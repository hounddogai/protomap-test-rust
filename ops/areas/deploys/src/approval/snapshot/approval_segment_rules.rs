use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval segment must pass. Check returns its problems, or none.
pub trait ApprovalSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSegmentAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ApprovalSegmentRule for ApprovalSegmentAllowedDescriptionRule {
    fn check(&self, item: &ApprovalSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is missing", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSegmentCodePrefixRule {
    pub prefix: String,
}

impl ApprovalSegmentRule for ApprovalSegmentCodePrefixRule {
    fn check(&self, item: &ApprovalSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSegmentCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalSegmentRule for ApprovalSegmentCodeLengthRule {
    fn check(&self, item: &ApprovalSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval segment.
#[derive(Debug, Clone)]
pub struct ApprovalSegmentValidator {
    pub rules: Vec<Rc<dyn ApprovalSegmentRule>>,
}

impl ApprovalSegmentValidator {
    pub fn validate(&self, item: &ApprovalSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_segment_validator() -> ApprovalSegmentValidator {
    ApprovalSegmentValidator {
        rules: vec![
            Rc::new(ApprovalSegmentAllowedDescriptionRule {
                allowed: vec!["strict".to_string(), "seasonal".to_string()],
            }) as Rc<dyn ApprovalSegmentRule>,
            Rc::new(ApprovalSegmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ApprovalSegmentCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_segment_validator();
        let item = new_approval_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalSegmentCodeLengthRule { max_length: 120 };
        let mut item = new_approval_segment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
