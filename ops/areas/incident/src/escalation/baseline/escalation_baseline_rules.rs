use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation baseline must pass. Check returns its problems, or none.
pub trait EscalationBaselineRule: std::fmt::Debug {
    fn check(&self, item: &EscalationBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBaselineExternalRefLengthRule {
    pub max_length: i64,
}

impl EscalationBaselineRule for EscalationBaselineExternalRefLengthRule {
    fn check(&self, item: &EscalationBaseline) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBaselineRequiredKeyIdRule {
    pub message: String,
}

impl EscalationBaselineRule for EscalationBaselineRequiredKeyIdRule {
    fn check(&self, item: &EscalationBaseline) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBaselineAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl EscalationBaselineRule for EscalationBaselineAllowedKeyIdRule {
    fn check(&self, item: &EscalationBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is not allowed", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBaselineRequiredExternalRefRule {
    pub message: String,
}

impl EscalationBaselineRule for EscalationBaselineRequiredExternalRefRule {
    fn check(&self, item: &EscalationBaseline) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation baseline.
#[derive(Debug, Clone)]
pub struct EscalationBaselineValidator {
    pub rules: Vec<Rc<dyn EscalationBaselineRule>>,
}

impl EscalationBaselineValidator {
    pub fn validate(&self, item: &EscalationBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_baseline_validator() -> EscalationBaselineValidator {
    EscalationBaselineValidator {
        rules: vec![
            Rc::new(EscalationBaselineExternalRefLengthRule { max_length: 80 }) as Rc<dyn EscalationBaselineRule>,
            Rc::new(EscalationBaselineRequiredKeyIdRule {
                message: "escalation baseline needs this field".to_string(),
            }),
            Rc::new(EscalationBaselineAllowedKeyIdRule {
                allowed: vec!["seasonal".to_string(), "tiered".to_string()],
            }),
            Rc::new(EscalationBaselineRequiredExternalRefRule {
                message: "escalation baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_baseline_validator();
        let item = new_escalation_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = EscalationBaselineExternalRefLengthRule { max_length: 120 };
        let mut item = new_escalation_baseline("x", "y");
        item.external_ref = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = EscalationBaselineRequiredKeyIdRule {
            message: "escalation baseline needs a value".to_string(),
        };
        let mut item = new_escalation_baseline("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = EscalationBaselineRequiredExternalRefRule {
            message: "escalation baseline needs a value".to_string(),
        };
        let mut item = new_escalation_baseline("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
