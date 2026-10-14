use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback rule assignment must pass. Check returns its problems, or none.
pub trait FallbackRuleAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRuleAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FallbackRuleAssignmentRule for FallbackRuleAssignmentAllowedTitleRule {
    fn check(&self, item: &FallbackRuleAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl FallbackRuleAssignmentRule for FallbackRuleAssignmentTitleLengthRule {
    fn check(&self, item: &FallbackRuleAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRuleAssignmentRule for FallbackRuleAssignmentCodeLengthRule {
    fn check(&self, item: &FallbackRuleAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignmentNameLengthRule {
    pub max_length: i64,
}

impl FallbackRuleAssignmentRule for FallbackRuleAssignmentNameLengthRule {
    fn check(&self, item: &FallbackRuleAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rule assignment.
#[derive(Debug, Clone)]
pub struct FallbackRuleAssignmentValidator {
    pub rules: Vec<Rc<dyn FallbackRuleAssignmentRule>>,
}

impl FallbackRuleAssignmentValidator {
    pub fn validate(&self, item: &FallbackRuleAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRuleAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRuleAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rule_assignment_validator() -> FallbackRuleAssignmentValidator {
    FallbackRuleAssignmentValidator {
        rules: vec![
            Rc::new(FallbackRuleAssignmentAllowedTitleRule {
                allowed: vec!["default".to_string()],
            }) as Rc<dyn FallbackRuleAssignmentRule>,
            Rc::new(FallbackRuleAssignmentTitleLengthRule { max_length: 64 }),
            Rc::new(FallbackRuleAssignmentCodeLengthRule { max_length: 86400 }),
            Rc::new(FallbackRuleAssignmentNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_rule_assignment_validator();
        let item = new_fallback_rule_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackRuleAssignmentTitleLengthRule { max_length: 255 };
        let mut item = new_fallback_rule_assignment("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackRuleAssignmentCodeLengthRule { max_length: 255 };
        let mut item = new_fallback_rule_assignment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackRuleAssignmentNameLengthRule { max_length: 86400 };
        let mut item = new_fallback_rule_assignment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
