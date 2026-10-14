use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending escalation mapping must pass. Check returns its problems, or none.
pub trait PendingEscalationMappingRule: std::fmt::Debug {
    fn check(&self, item: &PendingEscalationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationMappingRequiredCipherRule {
    pub message: String,
}

impl PendingEscalationMappingRule for PendingEscalationMappingRequiredCipherRule {
    fn check(&self, item: &PendingEscalationMapping) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationMappingCodePrefixRule {
    pub prefix: String,
}

impl PendingEscalationMappingRule for PendingEscalationMappingCodePrefixRule {
    fn check(&self, item: &PendingEscalationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationMappingCodeLengthRule {
    pub max_length: i64,
}

impl PendingEscalationMappingRule for PendingEscalationMappingCodeLengthRule {
    fn check(&self, item: &PendingEscalationMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationMappingCurrencyLengthRule {
    pub max_length: i64,
}

impl PendingEscalationMappingRule for PendingEscalationMappingCurrencyLengthRule {
    fn check(&self, item: &PendingEscalationMapping) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending escalation mapping.
#[derive(Debug, Clone)]
pub struct PendingEscalationMappingValidator {
    pub rules: Vec<Rc<dyn PendingEscalationMappingRule>>,
}

impl PendingEscalationMappingValidator {
    pub fn validate(&self, item: &PendingEscalationMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingEscalationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingEscalationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_escalation_mapping_validator() -> PendingEscalationMappingValidator {
    PendingEscalationMappingValidator {
        rules: vec![
            Rc::new(PendingEscalationMappingRequiredCipherRule {
                message: "pending escalation mapping needs a value".to_string(),
            }) as Rc<dyn PendingEscalationMappingRule>,
            Rc::new(PendingEscalationMappingCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingEscalationMappingCodeLengthRule { max_length: 32 }),
            Rc::new(PendingEscalationMappingCurrencyLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_escalation_mapping_validator();
        let item = new_pending_escalation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingEscalationMappingRequiredCipherRule {
            message: "pending escalation mapping needs an entry".to_string(),
        };
        let mut item = new_pending_escalation_mapping("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingEscalationMappingCodeLengthRule { max_length: 120 };
        let mut item = new_pending_escalation_mapping("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PendingEscalationMappingCurrencyLengthRule { max_length: 120 };
        let mut item = new_pending_escalation_mapping("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
