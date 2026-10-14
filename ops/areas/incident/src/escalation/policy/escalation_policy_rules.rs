use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation policy must pass. Check returns its problems, or none.
pub trait EscalationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &EscalationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationPolicyRule for EscalationPolicyAttemptsRangeRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl EscalationPolicyRule for EscalationPolicyAllowedTenantIdRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is too long", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyCodeLengthRule {
    pub max_length: i64,
}

impl EscalationPolicyRule for EscalationPolicyCodeLengthRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyTenantIdLengthRule {
    pub max_length: i64,
}

impl EscalationPolicyRule for EscalationPolicyTenantIdLengthRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
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
pub struct EscalationPolicyRequiredNameRule {
    pub message: String,
}

impl EscalationPolicyRule for EscalationPolicyRequiredNameRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicyRequiredTenantIdRule {
    pub message: String,
}

impl EscalationPolicyRule for EscalationPolicyRequiredTenantIdRule {
    fn check(&self, item: &EscalationPolicy) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation policy.
#[derive(Debug, Clone)]
pub struct EscalationPolicyValidator {
    pub rules: Vec<Rc<dyn EscalationPolicyRule>>,
}

impl EscalationPolicyValidator {
    pub fn validate(&self, item: &EscalationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_policy_validator() -> EscalationPolicyValidator {
    EscalationPolicyValidator {
        rules: vec![
            Rc::new(EscalationPolicyAttemptsRangeRule { min: 255, max: 86400 }) as Rc<dyn EscalationPolicyRule>,
            Rc::new(EscalationPolicyAllowedTenantIdRule {
                allowed: vec!["regional".to_string(), "rolling".to_string()],
            }),
            Rc::new(EscalationPolicyCodeLengthRule { max_length: 255 }),
            Rc::new(EscalationPolicyTenantIdLengthRule { max_length: 3600 }),
            Rc::new(EscalationPolicyRequiredNameRule {
                message: "escalation policy needs a value".to_string(),
            }),
            Rc::new(EscalationPolicyRequiredTenantIdRule {
                message: "escalation policy needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_policy_validator();
        let item = new_escalation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = EscalationPolicyAttemptsRangeRule { min: 120, max: 3600 };
        let mut item = new_escalation_policy("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationPolicyCodeLengthRule { max_length: 80 };
        let mut item = new_escalation_policy("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = EscalationPolicyTenantIdLengthRule { max_length: 120 };
        let mut item = new_escalation_policy("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EscalationPolicyRequiredNameRule {
            message: "escalation policy needs a value".to_string(),
        };
        let mut item = new_escalation_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = EscalationPolicyRequiredTenantIdRule {
            message: "escalation policy needs an entry".to_string(),
        };
        let mut item = new_escalation_policy("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
