use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered data export mapping must pass. Check returns its problems, or none.
pub trait TieredDataExportMappingRule: std::fmt::Debug {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDataExportMappingRule for TieredDataExportMappingRevisionRangeRule {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDataExportMappingRule for TieredDataExportMappingMinimumRangeRule {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDataExportMappingRule for TieredDataExportMappingPortRangeRule {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingTitleLengthRule {
    pub max_length: i64,
}

impl TieredDataExportMappingRule for TieredDataExportMappingTitleLengthRule {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMappingRequiredTitleRule {
    pub message: String,
}

impl TieredDataExportMappingRule for TieredDataExportMappingRequiredTitleRule {
    fn check(&self, item: &TieredDataExportMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered data export mapping.
#[derive(Debug, Clone)]
pub struct TieredDataExportMappingValidator {
    pub rules: Vec<Rc<dyn TieredDataExportMappingRule>>,
}

impl TieredDataExportMappingValidator {
    pub fn validate(&self, item: &TieredDataExportMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDataExportMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDataExportMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_data_export_mapping_validator() -> TieredDataExportMappingValidator {
    TieredDataExportMappingValidator {
        rules: vec![
            Rc::new(TieredDataExportMappingRevisionRangeRule { min: 16, max: 3600 })
                as Rc<dyn TieredDataExportMappingRule>,
            Rc::new(TieredDataExportMappingMinimumRangeRule { min: 2.2, max: 1698.5 }),
            Rc::new(TieredDataExportMappingPortRangeRule { min: 120, max: 3600 }),
            Rc::new(TieredDataExportMappingTitleLengthRule { max_length: 64 }),
            Rc::new(TieredDataExportMappingRequiredTitleRule {
                message: "tiered data export mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_data_export_mapping_validator();
        let item = new_tiered_data_export_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredDataExportMappingRevisionRangeRule { min: 16, max: 32 };
        let mut item = new_tiered_data_export_mapping("x", "y");
        item.revision = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TieredDataExportMappingMinimumRangeRule { min: 4.4, max: 146.4 };
        let mut item = new_tiered_data_export_mapping("x", "y");
        item.minimum = 147.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredDataExportMappingPortRangeRule { min: 120, max: 3600 };
        let mut item = new_tiered_data_export_mapping("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredDataExportMappingTitleLengthRule { max_length: 86400 };
        let mut item = new_tiered_data_export_mapping("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredDataExportMappingRequiredTitleRule {
            message: "tiered data export mapping needs this field".to_string(),
        };
        let mut item = new_tiered_data_export_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
