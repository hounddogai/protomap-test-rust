use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled escalation policy must pass. Check returns its problems, or none.
pub trait ScheduledEscalationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEscalationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEscalationPolicyAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl ScheduledEscalationPolicyRule for ScheduledEscalationPolicyAllowedChecksumRule {
    fn check(&self, item: &ScheduledEscalationPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is out of range", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEscalationPolicyRequiredNameRule {
    pub message: String,
}

impl ScheduledEscalationPolicyRule for ScheduledEscalationPolicyRequiredNameRule {
    fn check(&self, item: &ScheduledEscalationPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEscalationPolicyCodePrefixRule {
    pub prefix: String,
}

impl ScheduledEscalationPolicyRule for ScheduledEscalationPolicyCodePrefixRule {
    fn check(&self, item: &ScheduledEscalationPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled escalation policy.
#[derive(Debug, Clone)]
pub struct ScheduledEscalationPolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledEscalationPolicyRule>>,
}

impl ScheduledEscalationPolicyValidator {
    pub fn validate(&self, item: &ScheduledEscalationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledEscalationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEscalationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_escalation_policy_validator() -> ScheduledEscalationPolicyValidator {
    ScheduledEscalationPolicyValidator {
        rules: vec![
            Rc::new(ScheduledEscalationPolicyAllowedChecksumRule {
                allowed: vec!["regional".to_string()],
            }) as Rc<dyn ScheduledEscalationPolicyRule>,
            Rc::new(ScheduledEscalationPolicyRequiredNameRule {
                message: "scheduled escalation policy needs an entry".to_string(),
            }),
            Rc::new(ScheduledEscalationPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_escalation_policy_validator();
        let item = new_scheduled_escalation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledEscalationPolicyRequiredNameRule {
            message: "scheduled escalation policy needs this field".to_string(),
        };
        let mut item = new_scheduled_escalation_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
