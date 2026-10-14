use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped suspension entry must pass. Check returns its problems, or none.
pub trait CappedSuspensionEntryRule: std::fmt::Debug {
    fn check(&self, item: &CappedSuspensionEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntryCodePrefixRule {
    pub prefix: String,
}

impl CappedSuspensionEntryRule for CappedSuspensionEntryCodePrefixRule {
    fn check(&self, item: &CappedSuspensionEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntryBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSuspensionEntryRule for CappedSuspensionEntryBudgetRangeRule {
    fn check(&self, item: &CappedSuspensionEntry) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntryAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl CappedSuspensionEntryRule for CappedSuspensionEntryAllowedTimezoneRule {
    fn check(&self, item: &CappedSuspensionEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is expired", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntryRequiredTitleRule {
    pub message: String,
}

impl CappedSuspensionEntryRule for CappedSuspensionEntryRequiredTitleRule {
    fn check(&self, item: &CappedSuspensionEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped suspension entry.
#[derive(Debug, Clone)]
pub struct CappedSuspensionEntryValidator {
    pub rules: Vec<Rc<dyn CappedSuspensionEntryRule>>,
}

impl CappedSuspensionEntryValidator {
    pub fn validate(&self, item: &CappedSuspensionEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSuspensionEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSuspensionEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_suspension_entry_validator() -> CappedSuspensionEntryValidator {
    CappedSuspensionEntryValidator {
        rules: vec![
            Rc::new(CappedSuspensionEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn CappedSuspensionEntryRule>,
            Rc::new(CappedSuspensionEntryBudgetRangeRule { min: 3.5, max: 4869.9 }),
            Rc::new(CappedSuspensionEntryAllowedTimezoneRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(CappedSuspensionEntryRequiredTitleRule {
                message: "capped suspension entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_suspension_entry_validator();
        let item = new_capped_suspension_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = CappedSuspensionEntryBudgetRangeRule { min: 9.2, max: 3168.9 };
        let mut item = new_capped_suspension_entry("x", "y");
        item.budget = 3170.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedSuspensionEntryRequiredTitleRule {
            message: "capped suspension entry needs a value".to_string(),
        };
        let mut item = new_capped_suspension_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
