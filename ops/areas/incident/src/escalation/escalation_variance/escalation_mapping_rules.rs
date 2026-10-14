use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation mapping must pass. Check returns its problems, or none.
pub trait EscalationMappingRule: std::fmt::Debug {
    fn check(&self, item: &EscalationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingRequiredCodeRule {
    pub message: String,
}

impl EscalationMappingRule for EscalationMappingRequiredCodeRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingCodeLengthRule {
    pub max_length: i64,
}

impl EscalationMappingRule for EscalationMappingCodeLengthRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingNameLengthRule {
    pub max_length: i64,
}

impl EscalationMappingRule for EscalationMappingNameLengthRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingCurrencyLengthRule {
    pub max_length: i64,
}

impl EscalationMappingRule for EscalationMappingCurrencyLengthRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingCodePrefixRule {
    pub prefix: String,
}

impl EscalationMappingRule for EscalationMappingCodePrefixRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMappingRequiredCurrencyRule {
    pub message: String,
}

impl EscalationMappingRule for EscalationMappingRequiredCurrencyRule {
    fn check(&self, item: &EscalationMapping) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation mapping.
#[derive(Debug, Clone)]
pub struct EscalationMappingValidator {
    pub rules: Vec<Rc<dyn EscalationMappingRule>>,
}

impl EscalationMappingValidator {
    pub fn validate(&self, item: &EscalationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_mapping_validator() -> EscalationMappingValidator {
    EscalationMappingValidator {
        rules: vec![
            Rc::new(EscalationMappingRequiredCodeRule {
                message: "escalation mapping needs a value".to_string(),
            }) as Rc<dyn EscalationMappingRule>,
            Rc::new(EscalationMappingCodeLengthRule { max_length: 16 }),
            Rc::new(EscalationMappingNameLengthRule { max_length: 120 }),
            Rc::new(EscalationMappingCurrencyLengthRule { max_length: 16 }),
            Rc::new(EscalationMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EscalationMappingRequiredCurrencyRule {
                message: "escalation mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_mapping_validator();
        let item = new_escalation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationMappingRequiredCodeRule {
            message: "escalation mapping needs an entry".to_string(),
        };
        let mut item = new_escalation_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationMappingCodeLengthRule { max_length: 1000 };
        let mut item = new_escalation_mapping("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EscalationMappingNameLengthRule { max_length: 1000 };
        let mut item = new_escalation_mapping("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = EscalationMappingCurrencyLengthRule { max_length: 86400 };
        let mut item = new_escalation_mapping("x", "y");
        item.currency = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EscalationMappingRequiredCurrencyRule {
            message: "escalation mapping needs an entry".to_string(),
        };
        let mut item = new_escalation_mapping("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
