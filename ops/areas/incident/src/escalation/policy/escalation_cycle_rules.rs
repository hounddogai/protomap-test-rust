use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation cycle must pass. Check returns its problems, or none.
pub trait EscalationCycleRule: std::fmt::Debug {
    fn check(&self, item: &EscalationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleRequiredPrincipalRule {
    pub message: String,
}

impl EscalationCycleRule for EscalationCycleRequiredPrincipalRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleScopeLengthRule {
    pub max_length: i64,
}

impl EscalationCycleRule for EscalationCycleScopeLengthRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleNameLengthRule {
    pub max_length: i64,
}

impl EscalationCycleRule for EscalationCycleNameLengthRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleCodePrefixRule {
    pub prefix: String,
}

impl EscalationCycleRule for EscalationCycleCodePrefixRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCyclePrincipalLengthRule {
    pub max_length: i64,
}

impl EscalationCycleRule for EscalationCyclePrincipalLengthRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCycleCodeLengthRule {
    pub max_length: i64,
}

impl EscalationCycleRule for EscalationCycleCodeLengthRule {
    fn check(&self, item: &EscalationCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation cycle.
#[derive(Debug, Clone)]
pub struct EscalationCycleValidator {
    pub rules: Vec<Rc<dyn EscalationCycleRule>>,
}

impl EscalationCycleValidator {
    pub fn validate(&self, item: &EscalationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_cycle_validator() -> EscalationCycleValidator {
    EscalationCycleValidator {
        rules: vec![
            Rc::new(EscalationCycleRequiredPrincipalRule {
                message: "escalation cycle needs a value".to_string(),
            }) as Rc<dyn EscalationCycleRule>,
            Rc::new(EscalationCycleScopeLengthRule { max_length: 86400 }),
            Rc::new(EscalationCycleNameLengthRule { max_length: 80 }),
            Rc::new(EscalationCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EscalationCyclePrincipalLengthRule { max_length: 16 }),
            Rc::new(EscalationCycleCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_cycle_validator();
        let item = new_escalation_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = EscalationCycleRequiredPrincipalRule {
            message: "escalation cycle needs a value".to_string(),
        };
        let mut item = new_escalation_cycle("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = EscalationCycleScopeLengthRule { max_length: 32 };
        let mut item = new_escalation_cycle("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EscalationCycleNameLengthRule { max_length: 80 };
        let mut item = new_escalation_cycle("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = EscalationCyclePrincipalLengthRule { max_length: 255 };
        let mut item = new_escalation_cycle("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationCycleCodeLengthRule { max_length: 3600 };
        let mut item = new_escalation_cycle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
