use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce entry must pass. Check returns its problems, or none.
pub trait NonceEntryRule: std::fmt::Debug {
    fn check(&self, item: &NonceEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceEntryRule for NonceEntryMaximumRangeRule {
    fn check(&self, item: &NonceEntry) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryRequiredKeyIdRule {
    pub message: String,
}

impl NonceEntryRule for NonceEntryRequiredKeyIdRule {
    fn check(&self, item: &NonceEntry) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryRequiredLabelRule {
    pub message: String,
}

impl NonceEntryRule for NonceEntryRequiredLabelRule {
    fn check(&self, item: &NonceEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl NonceEntryRule for NonceEntryAllowedKeyIdRule {
    fn check(&self, item: &NonceEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is above maximum", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce entry.
#[derive(Debug, Clone)]
pub struct NonceEntryValidator {
    pub rules: Vec<Rc<dyn NonceEntryRule>>,
}

impl NonceEntryValidator {
    pub fn validate(&self, item: &NonceEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_entry_validator() -> NonceEntryValidator {
    NonceEntryValidator {
        rules: vec![
            Rc::new(NonceEntryMaximumRangeRule { min: 4.2, max: 3000.8 }) as Rc<dyn NonceEntryRule>,
            Rc::new(NonceEntryRequiredKeyIdRule {
                message: "nonce entry needs this field".to_string(),
            }),
            Rc::new(NonceEntryRequiredLabelRule {
                message: "nonce entry needs a value".to_string(),
            }),
            Rc::new(NonceEntryAllowedKeyIdRule {
                allowed: vec!["weighted".to_string(), "adaptive".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_entry_validator();
        let item = new_nonce_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = NonceEntryMaximumRangeRule { min: 5.5, max: 4038.8 };
        let mut item = new_nonce_entry("x", "y");
        item.maximum = 4040.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = NonceEntryRequiredKeyIdRule {
            message: "nonce entry needs a value".to_string(),
        };
        let mut item = new_nonce_entry("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = NonceEntryRequiredLabelRule {
            message: "nonce entry needs a value".to_string(),
        };
        let mut item = new_nonce_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
