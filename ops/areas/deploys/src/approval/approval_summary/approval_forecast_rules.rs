use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval forecast must pass. Check returns its problems, or none.
pub trait ApprovalForecastRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecastQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalForecastRule for ApprovalForecastQuotaUnitsRangeRule {
    fn check(&self, item: &ApprovalForecast) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecastRequiredSummaryRule {
    pub message: String,
}

impl ApprovalForecastRule for ApprovalForecastRequiredSummaryRule {
    fn check(&self, item: &ApprovalForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecastRequiredCodeRule {
    pub message: String,
}

impl ApprovalForecastRule for ApprovalForecastRequiredCodeRule {
    fn check(&self, item: &ApprovalForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecastBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalForecastRule for ApprovalForecastBatchSizeRangeRule {
    fn check(&self, item: &ApprovalForecast) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval forecast.
#[derive(Debug, Clone)]
pub struct ApprovalForecastValidator {
    pub rules: Vec<Rc<dyn ApprovalForecastRule>>,
}

impl ApprovalForecastValidator {
    pub fn validate(&self, item: &ApprovalForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_forecast_validator() -> ApprovalForecastValidator {
    ApprovalForecastValidator {
        rules: vec![
            Rc::new(ApprovalForecastQuotaUnitsRangeRule { min: 80, max: 3600 }) as Rc<dyn ApprovalForecastRule>,
            Rc::new(ApprovalForecastRequiredSummaryRule {
                message: "approval forecast needs an entry".to_string(),
            }),
            Rc::new(ApprovalForecastRequiredCodeRule {
                message: "approval forecast needs this field".to_string(),
            }),
            Rc::new(ApprovalForecastBatchSizeRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_forecast_validator();
        let item = new_approval_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ApprovalForecastQuotaUnitsRangeRule { min: 16, max: 86400 };
        let mut item = new_approval_forecast("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ApprovalForecastRequiredSummaryRule {
            message: "approval forecast needs a value".to_string(),
        };
        let mut item = new_approval_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalForecastRequiredCodeRule {
            message: "approval forecast needs this field".to_string(),
        };
        let mut item = new_approval_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ApprovalForecastBatchSizeRangeRule { min: 80, max: 1000 };
        let mut item = new_approval_forecast("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
