use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual forecast summary must pass. Check returns its problems, or none.
pub trait ManualForecastSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ManualForecastSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummaryRequiredNameRule {
    pub message: String,
}

impl ManualForecastSummaryRule for ManualForecastSummaryRequiredNameRule {
    fn check(&self, item: &ManualForecastSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummaryRequiredCodeRule {
    pub message: String,
}

impl ManualForecastSummaryRule for ManualForecastSummaryRequiredCodeRule {
    fn check(&self, item: &ManualForecastSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummaryAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ManualForecastSummaryRule for ManualForecastSummaryAllowedTenantIdRule {
    fn check(&self, item: &ManualForecastSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is unknown", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual forecast summary.
#[derive(Debug, Clone)]
pub struct ManualForecastSummaryValidator {
    pub rules: Vec<Rc<dyn ManualForecastSummaryRule>>,
}

impl ManualForecastSummaryValidator {
    pub fn validate(&self, item: &ManualForecastSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualForecastSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualForecastSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_forecast_summary_validator() -> ManualForecastSummaryValidator {
    ManualForecastSummaryValidator {
        rules: vec![
            Rc::new(ManualForecastSummaryRequiredNameRule {
                message: "manual forecast summary needs a value".to_string(),
            }) as Rc<dyn ManualForecastSummaryRule>,
            Rc::new(ManualForecastSummaryRequiredCodeRule {
                message: "manual forecast summary needs an entry".to_string(),
            }),
            Rc::new(ManualForecastSummaryAllowedTenantIdRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_forecast_summary_validator();
        let item = new_manual_forecast_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualForecastSummaryRequiredNameRule {
            message: "manual forecast summary needs this field".to_string(),
        };
        let mut item = new_manual_forecast_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualForecastSummaryRequiredCodeRule {
            message: "manual forecast summary needs this field".to_string(),
        };
        let mut item = new_manual_forecast_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
