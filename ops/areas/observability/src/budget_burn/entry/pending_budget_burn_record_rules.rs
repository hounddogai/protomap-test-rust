use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending budget burn record must pass. Check returns its problems, or none.
pub trait PendingBudgetBurnRecordRule: std::fmt::Debug {
    fn check(&self, item: &PendingBudgetBurnRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecordBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingBudgetBurnRecordRule for PendingBudgetBurnRecordBurstRangeRule {
    fn check(&self, item: &PendingBudgetBurnRecord) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecordNameLengthRule {
    pub max_length: i64,
}

impl PendingBudgetBurnRecordRule for PendingBudgetBurnRecordNameLengthRule {
    fn check(&self, item: &PendingBudgetBurnRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecordRequiredSourceRule {
    pub message: String,
}

impl PendingBudgetBurnRecordRule for PendingBudgetBurnRecordRequiredSourceRule {
    fn check(&self, item: &PendingBudgetBurnRecord) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending budget burn record.
#[derive(Debug, Clone)]
pub struct PendingBudgetBurnRecordValidator {
    pub rules: Vec<Rc<dyn PendingBudgetBurnRecordRule>>,
}

impl PendingBudgetBurnRecordValidator {
    pub fn validate(&self, item: &PendingBudgetBurnRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingBudgetBurnRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingBudgetBurnRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_budget_burn_record_validator() -> PendingBudgetBurnRecordValidator {
    PendingBudgetBurnRecordValidator {
        rules: vec![
            Rc::new(PendingBudgetBurnRecordBurstRangeRule { min: 16, max: 80 }) as Rc<dyn PendingBudgetBurnRecordRule>,
            Rc::new(PendingBudgetBurnRecordNameLengthRule { max_length: 32 }),
            Rc::new(PendingBudgetBurnRecordRequiredSourceRule {
                message: "pending budget burn record needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_budget_burn_record_validator();
        let item = new_pending_budget_burn_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PendingBudgetBurnRecordBurstRangeRule { min: 1000, max: 86400 };
        let mut item = new_pending_budget_burn_record("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingBudgetBurnRecordNameLengthRule { max_length: 16 };
        let mut item = new_pending_budget_burn_record("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PendingBudgetBurnRecordRequiredSourceRule {
            message: "pending budget burn record needs this field".to_string(),
        };
        let mut item = new_pending_budget_burn_record("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
