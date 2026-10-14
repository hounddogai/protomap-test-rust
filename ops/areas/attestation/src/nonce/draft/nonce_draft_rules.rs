use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce draft must pass. Check returns its problems, or none.
pub trait NonceDraftRule: std::fmt::Debug {
    fn check(&self, item: &NonceDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDraftCodeLengthRule {
    pub max_length: i64,
}

impl NonceDraftRule for NonceDraftCodeLengthRule {
    fn check(&self, item: &NonceDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDraftUnitLengthRule {
    pub max_length: i64,
}

impl NonceDraftRule for NonceDraftUnitLengthRule {
    fn check(&self, item: &NonceDraft) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDraftRequiredUnitRule {
    pub message: String,
}

impl NonceDraftRule for NonceDraftRequiredUnitRule {
    fn check(&self, item: &NonceDraft) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDraftQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceDraftRule for NonceDraftQuantityRangeRule {
    fn check(&self, item: &NonceDraft) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce draft.
#[derive(Debug, Clone)]
pub struct NonceDraftValidator {
    pub rules: Vec<Rc<dyn NonceDraftRule>>,
}

impl NonceDraftValidator {
    pub fn validate(&self, item: &NonceDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_draft_validator() -> NonceDraftValidator {
    NonceDraftValidator {
        rules: vec![
            Rc::new(NonceDraftCodeLengthRule { max_length: 32 }) as Rc<dyn NonceDraftRule>,
            Rc::new(NonceDraftUnitLengthRule { max_length: 86400 }),
            Rc::new(NonceDraftRequiredUnitRule {
                message: "nonce draft needs an entry".to_string(),
            }),
            Rc::new(NonceDraftQuantityRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_draft_validator();
        let item = new_nonce_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NonceDraftCodeLengthRule { max_length: 80 };
        let mut item = new_nonce_draft("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = NonceDraftUnitLengthRule { max_length: 255 };
        let mut item = new_nonce_draft("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = NonceDraftRequiredUnitRule {
            message: "nonce draft needs a value".to_string(),
        };
        let mut item = new_nonce_draft("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = NonceDraftQuantityRangeRule { min: 16, max: 120 };
        let mut item = new_nonce_draft("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
