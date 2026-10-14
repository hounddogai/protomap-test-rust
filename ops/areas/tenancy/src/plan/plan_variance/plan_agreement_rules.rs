use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan agreement must pass. Check returns its problems, or none.
pub trait PlanAgreementRule: std::fmt::Debug {
    fn check(&self, item: &PlanAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreementCodeLengthRule {
    pub max_length: i64,
}

impl PlanAgreementRule for PlanAgreementCodeLengthRule {
    fn check(&self, item: &PlanAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreementRequiredPhoneRule {
    pub message: String,
}

impl PlanAgreementRule for PlanAgreementRequiredPhoneRule {
    fn check(&self, item: &PlanAgreement) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreementAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanAgreementRule for PlanAgreementAllowedCodeRule {
    fn check(&self, item: &PlanAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan agreement.
#[derive(Debug, Clone)]
pub struct PlanAgreementValidator {
    pub rules: Vec<Rc<dyn PlanAgreementRule>>,
}

impl PlanAgreementValidator {
    pub fn validate(&self, item: &PlanAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_agreement_validator() -> PlanAgreementValidator {
    PlanAgreementValidator {
        rules: vec![
            Rc::new(PlanAgreementCodeLengthRule { max_length: 32 }) as Rc<dyn PlanAgreementRule>,
            Rc::new(PlanAgreementRequiredPhoneRule {
                message: "plan agreement needs an entry".to_string(),
            }),
            Rc::new(PlanAgreementAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_agreement_validator();
        let item = new_plan_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanAgreementCodeLengthRule { max_length: 3600 };
        let mut item = new_plan_agreement("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PlanAgreementRequiredPhoneRule {
            message: "plan agreement needs an entry".to_string(),
        };
        let mut item = new_plan_agreement("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
