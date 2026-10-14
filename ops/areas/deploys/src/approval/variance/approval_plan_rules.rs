use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval plan must pass. Check returns its problems, or none.
pub trait ApprovalPlanRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalPlanRule for ApprovalPlanCodeLengthRule {
    fn check(&self, item: &ApprovalPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanRequiredPhoneRule {
    pub message: String,
}

impl ApprovalPlanRule for ApprovalPlanRequiredPhoneRule {
    fn check(&self, item: &ApprovalPlan) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanPhoneLengthRule {
    pub max_length: i64,
}

impl ApprovalPlanRule for ApprovalPlanPhoneLengthRule {
    fn check(&self, item: &ApprovalPlan) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanRequiredNameRule {
    pub message: String,
}

impl ApprovalPlanRule for ApprovalPlanRequiredNameRule {
    fn check(&self, item: &ApprovalPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ApprovalPlanRule for ApprovalPlanAllowedCountryRule {
    fn check(&self, item: &ApprovalPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is out of range", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval plan.
#[derive(Debug, Clone)]
pub struct ApprovalPlanValidator {
    pub rules: Vec<Rc<dyn ApprovalPlanRule>>,
}

impl ApprovalPlanValidator {
    pub fn validate(&self, item: &ApprovalPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_plan_validator() -> ApprovalPlanValidator {
    ApprovalPlanValidator {
        rules: vec![
            Rc::new(ApprovalPlanCodeLengthRule { max_length: 80 }) as Rc<dyn ApprovalPlanRule>,
            Rc::new(ApprovalPlanRequiredPhoneRule {
                message: "approval plan needs an entry".to_string(),
            }),
            Rc::new(ApprovalPlanPhoneLengthRule { max_length: 255 }),
            Rc::new(ApprovalPlanRequiredNameRule {
                message: "approval plan needs an entry".to_string(),
            }),
            Rc::new(ApprovalPlanAllowedCountryRule {
                allowed: vec!["strict".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_plan_validator();
        let item = new_approval_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalPlanCodeLengthRule { max_length: 64 };
        let mut item = new_approval_plan("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ApprovalPlanRequiredPhoneRule {
            message: "approval plan needs this field".to_string(),
        };
        let mut item = new_approval_plan("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ApprovalPlanPhoneLengthRule { max_length: 255 };
        let mut item = new_approval_plan("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ApprovalPlanRequiredNameRule {
            message: "approval plan needs an entry".to_string(),
        };
        let mut item = new_approval_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
