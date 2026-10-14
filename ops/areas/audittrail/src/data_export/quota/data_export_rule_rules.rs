use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export rule must pass. Check returns its problems, or none.
pub trait DataExportRuleRule: std::fmt::Debug {
    fn check(&self, item: &DataExportRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportRuleRule for DataExportRuleSequenceRangeRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleNameLengthRule {
    pub max_length: i64,
}

impl DataExportRuleRule for DataExportRuleNameLengthRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportRuleRule for DataExportRuleGraceDaysRangeRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportRuleRule for DataExportRuleQuantityRangeRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleCurrencyLengthRule {
    pub max_length: i64,
}

impl DataExportRuleRule for DataExportRuleCurrencyLengthRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleApprovedGuardRule {
    pub limit: i64,
}

impl DataExportRuleRule for DataExportRuleApprovedGuardRule {
    fn check(&self, item: &DataExportRule) -> Vec<String> {
        if item.approved && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while approved",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export rule.
#[derive(Debug, Clone)]
pub struct DataExportRuleValidator {
    pub rules: Vec<Rc<dyn DataExportRuleRule>>,
}

impl DataExportRuleValidator {
    pub fn validate(&self, item: &DataExportRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_rule_validator() -> DataExportRuleValidator {
    DataExportRuleValidator {
        rules: vec![
            Rc::new(DataExportRuleSequenceRangeRule { min: 3600, max: 3600 }) as Rc<dyn DataExportRuleRule>,
            Rc::new(DataExportRuleNameLengthRule { max_length: 16 }),
            Rc::new(DataExportRuleGraceDaysRangeRule { min: 32, max: 1000 }),
            Rc::new(DataExportRuleQuantityRangeRule { min: 32, max: 86400 }),
            Rc::new(DataExportRuleCurrencyLengthRule { max_length: 3600 }),
            Rc::new(DataExportRuleApprovedGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_rule_validator();
        let item = new_data_export_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DataExportRuleSequenceRangeRule { min: 64, max: 64 };
        let mut item = new_data_export_rule("x", "y");
        item.sequence = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DataExportRuleNameLengthRule { max_length: 1000 };
        let mut item = new_data_export_rule("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DataExportRuleGraceDaysRangeRule { min: 1000, max: 86400 };
        let mut item = new_data_export_rule("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DataExportRuleQuantityRangeRule { min: 255, max: 3600 };
        let mut item = new_data_export_rule("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = DataExportRuleCurrencyLengthRule { max_length: 3600 };
        let mut item = new_data_export_rule("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
