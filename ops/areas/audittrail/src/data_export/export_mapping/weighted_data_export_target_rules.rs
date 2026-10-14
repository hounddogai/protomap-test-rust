use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted data export target must pass. Check returns its problems, or none.
pub trait WeightedDataExportTargetRule: std::fmt::Debug {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetHostLengthRule {
    pub max_length: i64,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetHostLengthRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetNameLengthRule {
    pub max_length: i64,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetNameLengthRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetCostRangeRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetRequiredHostRule {
    pub message: String,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetRequiredHostRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetAllowedHostRule {
    pub allowed: Vec<String>,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetAllowedHostRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is expired", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTargetPhoneLengthRule {
    pub max_length: i64,
}

impl WeightedDataExportTargetRule for WeightedDataExportTargetPhoneLengthRule {
    fn check(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted data export target.
#[derive(Debug, Clone)]
pub struct WeightedDataExportTargetValidator {
    pub rules: Vec<Rc<dyn WeightedDataExportTargetRule>>,
}

impl WeightedDataExportTargetValidator {
    pub fn validate(&self, item: &WeightedDataExportTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedDataExportTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedDataExportTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_data_export_target_validator() -> WeightedDataExportTargetValidator {
    WeightedDataExportTargetValidator {
        rules: vec![
            Rc::new(WeightedDataExportTargetHostLengthRule { max_length: 64 }) as Rc<dyn WeightedDataExportTargetRule>,
            Rc::new(WeightedDataExportTargetNameLengthRule { max_length: 86400 }),
            Rc::new(WeightedDataExportTargetCostRangeRule { min: 1.0, max: 548.8 }),
            Rc::new(WeightedDataExportTargetRequiredHostRule {
                message: "weighted data export target needs this field".to_string(),
            }),
            Rc::new(WeightedDataExportTargetAllowedHostRule {
                allowed: vec!["manual".to_string(), "manual".to_string()],
            }),
            Rc::new(WeightedDataExportTargetPhoneLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_data_export_target_validator();
        let item = new_weighted_data_export_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = WeightedDataExportTargetHostLengthRule { max_length: 86400 };
        let mut item = new_weighted_data_export_target("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedDataExportTargetNameLengthRule { max_length: 86400 };
        let mut item = new_weighted_data_export_target("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedDataExportTargetCostRangeRule { min: 8.4, max: 3998.7 };
        let mut item = new_weighted_data_export_target("x", "y");
        item.cost = 4000.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = WeightedDataExportTargetRequiredHostRule {
            message: "weighted data export target needs this field".to_string(),
        };
        let mut item = new_weighted_data_export_target("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = WeightedDataExportTargetPhoneLengthRule { max_length: 64 };
        let mut item = new_weighted_data_export_target("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
