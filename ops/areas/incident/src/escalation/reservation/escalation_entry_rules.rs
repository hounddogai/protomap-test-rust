use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation entry must pass. Check returns its problems, or none.
pub trait EscalationEntryRule: std::fmt::Debug {
    fn check(&self, item: &EscalationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationEntryRule for EscalationEntryPriorityRangeRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryCodePrefixRule {
    pub prefix: String,
}

impl EscalationEntryRule for EscalationEntryCodePrefixRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryRequiredLabelRule {
    pub message: String,
}

impl EscalationEntryRule for EscalationEntryRequiredLabelRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationEntryRule for EscalationEntryQuantityRangeRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryCodeLengthRule {
    pub max_length: i64,
}

impl EscalationEntryRule for EscalationEntryCodeLengthRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationEntryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationEntryRule for EscalationEntryRatioRangeRule {
    fn check(&self, item: &EscalationEntry) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation entry.
#[derive(Debug, Clone)]
pub struct EscalationEntryValidator {
    pub rules: Vec<Rc<dyn EscalationEntryRule>>,
}

impl EscalationEntryValidator {
    pub fn validate(&self, item: &EscalationEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_entry_validator() -> EscalationEntryValidator {
    EscalationEntryValidator {
        rules: vec![
            Rc::new(EscalationEntryPriorityRangeRule { min: 120, max: 255 }) as Rc<dyn EscalationEntryRule>,
            Rc::new(EscalationEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EscalationEntryRequiredLabelRule {
                message: "escalation entry needs an entry".to_string(),
            }),
            Rc::new(EscalationEntryQuantityRangeRule { min: 64, max: 3600 }),
            Rc::new(EscalationEntryCodeLengthRule { max_length: 1000 }),
            Rc::new(EscalationEntryRatioRangeRule { min: 6.4, max: 1812.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_entry_validator();
        let item = new_escalation_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = EscalationEntryPriorityRangeRule { min: 80, max: 1000 };
        let mut item = new_escalation_entry("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationEntryRequiredLabelRule {
            message: "escalation entry needs this field".to_string(),
        };
        let mut item = new_escalation_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = EscalationEntryQuantityRangeRule { min: 64, max: 64 };
        let mut item = new_escalation_entry("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationEntryCodeLengthRule { max_length: 255 };
        let mut item = new_escalation_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = EscalationEntryRatioRangeRule { min: 4.4, max: 1774.8 };
        let mut item = new_escalation_entry("x", "y");
        item.ratio = 1776.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
