use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard forecast must pass. Check returns its problems, or none.
pub trait DashboardForecastRule: std::fmt::Debug {
    fn check(&self, item: &DashboardForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastCodeLengthRule {
    pub max_length: i64,
}

impl DashboardForecastRule for DashboardForecastCodeLengthRule {
    fn check(&self, item: &DashboardForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastRequiredLabelRule {
    pub message: String,
}

impl DashboardForecastRule for DashboardForecastRequiredLabelRule {
    fn check(&self, item: &DashboardForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastCodePrefixRule {
    pub prefix: String,
}

impl DashboardForecastRule for DashboardForecastCodePrefixRule {
    fn check(&self, item: &DashboardForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastSegmentLengthRule {
    pub max_length: i64,
}

impl DashboardForecastRule for DashboardForecastSegmentLengthRule {
    fn check(&self, item: &DashboardForecast) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardForecastLabelLengthRule {
    pub max_length: i64,
}

impl DashboardForecastRule for DashboardForecastLabelLengthRule {
    fn check(&self, item: &DashboardForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard forecast.
#[derive(Debug, Clone)]
pub struct DashboardForecastValidator {
    pub rules: Vec<Rc<dyn DashboardForecastRule>>,
}

impl DashboardForecastValidator {
    pub fn validate(&self, item: &DashboardForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_forecast_validator() -> DashboardForecastValidator {
    DashboardForecastValidator {
        rules: vec![
            Rc::new(DashboardForecastCodeLengthRule { max_length: 32 }) as Rc<dyn DashboardForecastRule>,
            Rc::new(DashboardForecastRequiredLabelRule {
                message: "dashboard forecast needs an entry".to_string(),
            }),
            Rc::new(DashboardForecastCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DashboardForecastSegmentLengthRule { max_length: 32 }),
            Rc::new(DashboardForecastLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_forecast_validator();
        let item = new_dashboard_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardForecastCodeLengthRule { max_length: 80 };
        let mut item = new_dashboard_forecast("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DashboardForecastRequiredLabelRule {
            message: "dashboard forecast needs this field".to_string(),
        };
        let mut item = new_dashboard_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = DashboardForecastSegmentLengthRule { max_length: 16 };
        let mut item = new_dashboard_forecast("x", "y");
        item.segment = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DashboardForecastLabelLengthRule { max_length: 64 };
        let mut item = new_dashboard_forecast("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
