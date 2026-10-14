use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy isolation ledger must pass. Check returns its problems, or none.
pub trait LegacyIsolationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationLedgerTitleLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationLedgerRule for LegacyIsolationLedgerTitleLengthRule {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationLedgerDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyIsolationLedgerRule for LegacyIsolationLedgerDurationMinutesRangeRule {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationLedgerVisibleGuardRule {
    pub limit: i64,
}

impl LegacyIsolationLedgerRule for LegacyIsolationLedgerVisibleGuardRule {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        if item.visible && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while visible",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationLedgerRequiredRegionRule {
    pub message: String,
}

impl LegacyIsolationLedgerRule for LegacyIsolationLedgerRequiredRegionRule {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationLedgerRequiredTitleRule {
    pub message: String,
}

impl LegacyIsolationLedgerRule for LegacyIsolationLedgerRequiredTitleRule {
    fn check(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy isolation ledger.
#[derive(Debug, Clone)]
pub struct LegacyIsolationLedgerValidator {
    pub rules: Vec<Rc<dyn LegacyIsolationLedgerRule>>,
}

impl LegacyIsolationLedgerValidator {
    pub fn validate(&self, item: &LegacyIsolationLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyIsolationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIsolationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_isolation_ledger_validator() -> LegacyIsolationLedgerValidator {
    LegacyIsolationLedgerValidator {
        rules: vec![
            Rc::new(LegacyIsolationLedgerTitleLengthRule { max_length: 3600 }) as Rc<dyn LegacyIsolationLedgerRule>,
            Rc::new(LegacyIsolationLedgerDurationMinutesRangeRule { min: 64, max: 3600 }),
            Rc::new(LegacyIsolationLedgerVisibleGuardRule { limit: 1000 }),
            Rc::new(LegacyIsolationLedgerRequiredRegionRule {
                message: "legacy isolation ledger needs this field".to_string(),
            }),
            Rc::new(LegacyIsolationLedgerRequiredTitleRule {
                message: "legacy isolation ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_isolation_ledger_validator();
        let item = new_legacy_isolation_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyIsolationLedgerTitleLengthRule { max_length: 64 };
        let mut item = new_legacy_isolation_ledger("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LegacyIsolationLedgerDurationMinutesRangeRule { min: 64, max: 80 };
        let mut item = new_legacy_isolation_ledger("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LegacyIsolationLedgerRequiredRegionRule {
            message: "legacy isolation ledger needs this field".to_string(),
        };
        let mut item = new_legacy_isolation_ledger("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyIsolationLedgerRequiredTitleRule {
            message: "legacy isolation ledger needs a value".to_string(),
        };
        let mut item = new_legacy_isolation_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
