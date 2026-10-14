use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default data export estimate must pass. Check returns its problems, or none.
pub trait DefaultDataExportEstimateRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimateClusterLengthRule {
    pub max_length: i64,
}

impl DefaultDataExportEstimateRule for DefaultDataExportEstimateClusterLengthRule {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimateAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl DefaultDataExportEstimateRule for DefaultDataExportEstimateAllowedContactEmailRule {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is blocked", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimatePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultDataExportEstimateRule for DefaultDataExportEstimatePortRangeRule {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimateRequiredClusterRule {
    pub message: String,
}

impl DefaultDataExportEstimateRule for DefaultDataExportEstimateRequiredClusterRule {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimateRequiredCodeRule {
    pub message: String,
}

impl DefaultDataExportEstimateRule for DefaultDataExportEstimateRequiredCodeRule {
    fn check(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default data export estimate.
#[derive(Debug, Clone)]
pub struct DefaultDataExportEstimateValidator {
    pub rules: Vec<Rc<dyn DefaultDataExportEstimateRule>>,
}

impl DefaultDataExportEstimateValidator {
    pub fn validate(&self, item: &DefaultDataExportEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultDataExportEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDataExportEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_data_export_estimate_validator() -> DefaultDataExportEstimateValidator {
    DefaultDataExportEstimateValidator {
        rules: vec![
            Rc::new(DefaultDataExportEstimateClusterLengthRule { max_length: 32 })
                as Rc<dyn DefaultDataExportEstimateRule>,
            Rc::new(DefaultDataExportEstimateAllowedContactEmailRule {
                allowed: vec!["global".to_string(), "batched".to_string()],
            }),
            Rc::new(DefaultDataExportEstimatePortRangeRule { min: 120, max: 3600 }),
            Rc::new(DefaultDataExportEstimateRequiredClusterRule {
                message: "default data export estimate needs this field".to_string(),
            }),
            Rc::new(DefaultDataExportEstimateRequiredCodeRule {
                message: "default data export estimate needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_data_export_estimate_validator();
        let item = new_default_data_export_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = DefaultDataExportEstimateClusterLengthRule { max_length: 32 };
        let mut item = new_default_data_export_estimate("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = DefaultDataExportEstimatePortRangeRule { min: 1000, max: 3600 };
        let mut item = new_default_data_export_estimate("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = DefaultDataExportEstimateRequiredClusterRule {
            message: "default data export estimate needs a value".to_string(),
        };
        let mut item = new_default_data_export_estimate("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultDataExportEstimateRequiredCodeRule {
            message: "default data export estimate needs an entry".to_string(),
        };
        let mut item = new_default_data_export_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
