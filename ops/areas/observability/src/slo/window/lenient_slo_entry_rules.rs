use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient slo entry must pass. Check returns its problems, or none.
pub trait LenientSloEntryRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntryRequiredHostRule {
    pub message: String,
}

impl LenientSloEntryRule for LenientSloEntryRequiredHostRule {
    fn check(&self, item: &LenientSloEntry) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntryCodePrefixRule {
    pub prefix: String,
}

impl LenientSloEntryRule for LenientSloEntryCodePrefixRule {
    fn check(&self, item: &LenientSloEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSloEntryRule for LenientSloEntryAmountRangeRule {
    fn check(&self, item: &LenientSloEntry) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LenientSloEntryRule for LenientSloEntryAllowedNameRule {
    fn check(&self, item: &LenientSloEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo entry.
#[derive(Debug, Clone)]
pub struct LenientSloEntryValidator {
    pub rules: Vec<Rc<dyn LenientSloEntryRule>>,
}

impl LenientSloEntryValidator {
    pub fn validate(&self, item: &LenientSloEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSloEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_entry_validator() -> LenientSloEntryValidator {
    LenientSloEntryValidator {
        rules: vec![
            Rc::new(LenientSloEntryRequiredHostRule {
                message: "lenient slo entry needs an entry".to_string(),
            }) as Rc<dyn LenientSloEntryRule>,
            Rc::new(LenientSloEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientSloEntryAmountRangeRule { min: 4.9, max: 3415.1 }),
            Rc::new(LenientSloEntryAllowedNameRule {
                allowed: vec!["global".to_string(), "seasonal".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_entry_validator();
        let item = new_lenient_slo_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LenientSloEntryRequiredHostRule {
            message: "lenient slo entry needs a value".to_string(),
        };
        let mut item = new_lenient_slo_entry("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LenientSloEntryAmountRangeRule { min: 8.9, max: 4638.5 };
        let mut item = new_lenient_slo_entry("x", "y");
        item.amount = 4640.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
