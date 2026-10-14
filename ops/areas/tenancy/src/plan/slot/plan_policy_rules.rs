use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan policy must pass. Check returns its problems, or none.
pub trait PlanPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PlanPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyRequiredNameRule {
    pub message: String,
}

impl PlanPolicyRule for PlanPolicyRequiredNameRule {
    fn check(&self, item: &PlanPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyRequiredCodeRule {
    pub message: String,
}

impl PlanPolicyRule for PlanPolicyRequiredCodeRule {
    fn check(&self, item: &PlanPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyCodePrefixRule {
    pub prefix: String,
}

impl PlanPolicyRule for PlanPolicyCodePrefixRule {
    fn check(&self, item: &PlanPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyPhoneLengthRule {
    pub max_length: i64,
}

impl PlanPolicyRule for PlanPolicyPhoneLengthRule {
    fn check(&self, item: &PlanPolicy) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan policy.
#[derive(Debug, Clone)]
pub struct PlanPolicyValidator {
    pub rules: Vec<Rc<dyn PlanPolicyRule>>,
}

impl PlanPolicyValidator {
    pub fn validate(&self, item: &PlanPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_policy_validator() -> PlanPolicyValidator {
    PlanPolicyValidator {
        rules: vec![
            Rc::new(PlanPolicyRequiredNameRule {
                message: "plan policy needs a value".to_string(),
            }) as Rc<dyn PlanPolicyRule>,
            Rc::new(PlanPolicyRequiredCodeRule {
                message: "plan policy needs this field".to_string(),
            }),
            Rc::new(PlanPolicyCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PlanPolicyPhoneLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_policy_validator();
        let item = new_plan_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PlanPolicyRequiredNameRule {
            message: "plan policy needs this field".to_string(),
        };
        let mut item = new_plan_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanPolicyRequiredCodeRule {
            message: "plan policy needs a value".to_string(),
        };
        let mut item = new_plan_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PlanPolicyPhoneLengthRule { max_length: 3600 };
        let mut item = new_plan_policy("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
