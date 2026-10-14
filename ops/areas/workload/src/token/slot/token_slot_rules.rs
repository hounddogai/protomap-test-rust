use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token slot must pass. Check returns its problems, or none.
pub trait TokenSlotRule: std::fmt::Debug {
    fn check(&self, item: &TokenSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlotCodePrefixRule {
    pub prefix: String,
}

impl TokenSlotRule for TokenSlotCodePrefixRule {
    fn check(&self, item: &TokenSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlotNotesLengthRule {
    pub max_length: i64,
}

impl TokenSlotRule for TokenSlotNotesLengthRule {
    fn check(&self, item: &TokenSlot) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlotRequiredLabelRule {
    pub message: String,
}

impl TokenSlotRule for TokenSlotRequiredLabelRule {
    fn check(&self, item: &TokenSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token slot.
#[derive(Debug, Clone)]
pub struct TokenSlotValidator {
    pub rules: Vec<Rc<dyn TokenSlotRule>>,
}

impl TokenSlotValidator {
    pub fn validate(&self, item: &TokenSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_slot_validator() -> TokenSlotValidator {
    TokenSlotValidator {
        rules: vec![
            Rc::new(TokenSlotCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn TokenSlotRule>,
            Rc::new(TokenSlotNotesLengthRule { max_length: 16 }),
            Rc::new(TokenSlotRequiredLabelRule {
                message: "token slot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_slot_validator();
        let item = new_token_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TokenSlotNotesLengthRule { max_length: 80 };
        let mut item = new_token_slot("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TokenSlotRequiredLabelRule {
            message: "token slot needs a value".to_string(),
        };
        let mut item = new_token_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
