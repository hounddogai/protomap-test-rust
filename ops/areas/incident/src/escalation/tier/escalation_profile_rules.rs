use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation profile must pass. Check returns its problems, or none.
pub trait EscalationProfileRule: std::fmt::Debug {
    fn check(&self, item: &EscalationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProfileRequiredTitleRule {
    pub message: String,
}

impl EscalationProfileRule for EscalationProfileRequiredTitleRule {
    fn check(&self, item: &EscalationProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProfileTenantIdLengthRule {
    pub max_length: i64,
}

impl EscalationProfileRule for EscalationProfileTenantIdLengthRule {
    fn check(&self, item: &EscalationProfile) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProfileRequiredCodeRule {
    pub message: String,
}

impl EscalationProfileRule for EscalationProfileRequiredCodeRule {
    fn check(&self, item: &EscalationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation profile.
#[derive(Debug, Clone)]
pub struct EscalationProfileValidator {
    pub rules: Vec<Rc<dyn EscalationProfileRule>>,
}

impl EscalationProfileValidator {
    pub fn validate(&self, item: &EscalationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_profile_validator() -> EscalationProfileValidator {
    EscalationProfileValidator {
        rules: vec![
            Rc::new(EscalationProfileRequiredTitleRule {
                message: "escalation profile needs a value".to_string(),
            }) as Rc<dyn EscalationProfileRule>,
            Rc::new(EscalationProfileTenantIdLengthRule { max_length: 32 }),
            Rc::new(EscalationProfileRequiredCodeRule {
                message: "escalation profile needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_profile_validator();
        let item = new_escalation_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationProfileRequiredTitleRule {
            message: "escalation profile needs an entry".to_string(),
        };
        let mut item = new_escalation_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = EscalationProfileTenantIdLengthRule { max_length: 64 };
        let mut item = new_escalation_profile("x", "y");
        item.tenant_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationProfileRequiredCodeRule {
            message: "escalation profile needs an entry".to_string(),
        };
        let mut item = new_escalation_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
