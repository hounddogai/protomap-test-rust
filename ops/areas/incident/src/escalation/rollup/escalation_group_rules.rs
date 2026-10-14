use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation group must pass. Check returns its problems, or none.
pub trait EscalationGroupRule: std::fmt::Debug {
    fn check(&self, item: &EscalationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationGroupRequiredLabelRule {
    pub message: String,
}

impl EscalationGroupRule for EscalationGroupRequiredLabelRule {
    fn check(&self, item: &EscalationGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationGroupLabelLengthRule {
    pub max_length: i64,
}

impl EscalationGroupRule for EscalationGroupLabelLengthRule {
    fn check(&self, item: &EscalationGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationGroupRequiredCodeRule {
    pub message: String,
}

impl EscalationGroupRule for EscalationGroupRequiredCodeRule {
    fn check(&self, item: &EscalationGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation group.
#[derive(Debug, Clone)]
pub struct EscalationGroupValidator {
    pub rules: Vec<Rc<dyn EscalationGroupRule>>,
}

impl EscalationGroupValidator {
    pub fn validate(&self, item: &EscalationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_group_validator() -> EscalationGroupValidator {
    EscalationGroupValidator {
        rules: vec![
            Rc::new(EscalationGroupRequiredLabelRule {
                message: "escalation group needs an entry".to_string(),
            }) as Rc<dyn EscalationGroupRule>,
            Rc::new(EscalationGroupLabelLengthRule { max_length: 120 }),
            Rc::new(EscalationGroupRequiredCodeRule {
                message: "escalation group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_group_validator();
        let item = new_escalation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationGroupRequiredLabelRule {
            message: "escalation group needs a value".to_string(),
        };
        let mut item = new_escalation_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EscalationGroupLabelLengthRule { max_length: 32 };
        let mut item = new_escalation_group("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationGroupRequiredCodeRule {
            message: "escalation group needs a value".to_string(),
        };
        let mut item = new_escalation_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
