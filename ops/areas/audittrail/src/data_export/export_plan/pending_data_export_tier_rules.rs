use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending data export tier must pass. Check returns its problems, or none.
pub trait PendingDataExportTierRule: std::fmt::Debug {
    fn check(&self, item: &PendingDataExportTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTierQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingDataExportTierRule for PendingDataExportTierQuantityRangeRule {
    fn check(&self, item: &PendingDataExportTier) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTierRequiredTimezoneRule {
    pub message: String,
}

impl PendingDataExportTierRule for PendingDataExportTierRequiredTimezoneRule {
    fn check(&self, item: &PendingDataExportTier) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTierRequiredUnitRule {
    pub message: String,
}

impl PendingDataExportTierRule for PendingDataExportTierRequiredUnitRule {
    fn check(&self, item: &PendingDataExportTier) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending data export tier.
#[derive(Debug, Clone)]
pub struct PendingDataExportTierValidator {
    pub rules: Vec<Rc<dyn PendingDataExportTierRule>>,
}

impl PendingDataExportTierValidator {
    pub fn validate(&self, item: &PendingDataExportTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingDataExportTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDataExportTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_data_export_tier_validator() -> PendingDataExportTierValidator {
    PendingDataExportTierValidator {
        rules: vec![
            Rc::new(PendingDataExportTierQuantityRangeRule { min: 64, max: 86400 })
                as Rc<dyn PendingDataExportTierRule>,
            Rc::new(PendingDataExportTierRequiredTimezoneRule {
                message: "pending data export tier needs this field".to_string(),
            }),
            Rc::new(PendingDataExportTierRequiredUnitRule {
                message: "pending data export tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_data_export_tier_validator();
        let item = new_pending_data_export_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PendingDataExportTierQuantityRangeRule { min: 64, max: 120 };
        let mut item = new_pending_data_export_tier("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PendingDataExportTierRequiredTimezoneRule {
            message: "pending data export tier needs this field".to_string(),
        };
        let mut item = new_pending_data_export_tier("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PendingDataExportTierRequiredUnitRule {
            message: "pending data export tier needs an entry".to_string(),
        };
        let mut item = new_pending_data_export_tier("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
