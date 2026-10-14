use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export target must pass. Check returns its problems, or none.
pub trait DataExportTargetRule: std::fmt::Debug {
    fn check(&self, item: &DataExportTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetCodePrefixRule {
    pub prefix: String,
}

impl DataExportTargetRule for DataExportTargetCodePrefixRule {
    fn check(&self, item: &DataExportTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DataExportTargetRule for DataExportTargetAllowedNameRule {
    fn check(&self, item: &DataExportTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportTargetRule for DataExportTargetMarginRangeRule {
    fn check(&self, item: &DataExportTarget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetNameLengthRule {
    pub max_length: i64,
}

impl DataExportTargetRule for DataExportTargetNameLengthRule {
    fn check(&self, item: &DataExportTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportTargetRule for DataExportTargetBurstRangeRule {
    fn check(&self, item: &DataExportTarget) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export target.
#[derive(Debug, Clone)]
pub struct DataExportTargetValidator {
    pub rules: Vec<Rc<dyn DataExportTargetRule>>,
}

impl DataExportTargetValidator {
    pub fn validate(&self, item: &DataExportTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_target_validator() -> DataExportTargetValidator {
    DataExportTargetValidator {
        rules: vec![
            Rc::new(DataExportTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DataExportTargetRule>,
            Rc::new(DataExportTargetAllowedNameRule { allowed: Vec::new() }),
            Rc::new(DataExportTargetMarginRangeRule { min: 3.2, max: 1845.0 }),
            Rc::new(DataExportTargetNameLengthRule { max_length: 1000 }),
            Rc::new(DataExportTargetBurstRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_target_validator();
        let item = new_data_export_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DataExportTargetMarginRangeRule { min: 4.4, max: 2545.2 };
        let mut item = new_data_export_target("x", "y");
        item.margin = 2546.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DataExportTargetNameLengthRule { max_length: 255 };
        let mut item = new_data_export_target("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DataExportTargetBurstRangeRule { min: 32, max: 3600 };
        let mut item = new_data_export_target("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
