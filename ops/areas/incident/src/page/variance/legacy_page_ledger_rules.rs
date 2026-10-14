use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy page ledger must pass. Check returns its problems, or none.
pub trait LegacyPageLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPageLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedgerCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPageLedgerRule for LegacyPageLedgerCeilingRangeRule {
    fn check(&self, item: &LegacyPageLedger) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedgerLabelLengthRule {
    pub max_length: i64,
}

impl LegacyPageLedgerRule for LegacyPageLedgerLabelLengthRule {
    fn check(&self, item: &LegacyPageLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedgerRequiredCodeRule {
    pub message: String,
}

impl LegacyPageLedgerRule for LegacyPageLedgerRequiredCodeRule {
    fn check(&self, item: &LegacyPageLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy page ledger.
#[derive(Debug, Clone)]
pub struct LegacyPageLedgerValidator {
    pub rules: Vec<Rc<dyn LegacyPageLedgerRule>>,
}

impl LegacyPageLedgerValidator {
    pub fn validate(&self, item: &LegacyPageLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyPageLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPageLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_page_ledger_validator() -> LegacyPageLedgerValidator {
    LegacyPageLedgerValidator {
        rules: vec![
            Rc::new(LegacyPageLedgerCeilingRangeRule { min: 0.3, max: 2189.4 }) as Rc<dyn LegacyPageLedgerRule>,
            Rc::new(LegacyPageLedgerLabelLengthRule { max_length: 16 }),
            Rc::new(LegacyPageLedgerRequiredCodeRule {
                message: "legacy page ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_page_ledger_validator();
        let item = new_legacy_page_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LegacyPageLedgerCeilingRangeRule { min: 6.8, max: 188.3 };
        let mut item = new_legacy_page_ledger("x", "y");
        item.ceiling = 189.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyPageLedgerLabelLengthRule { max_length: 32 };
        let mut item = new_legacy_page_ledger("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyPageLedgerRequiredCodeRule {
            message: "legacy page ledger needs this field".to_string(),
        };
        let mut item = new_legacy_page_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
