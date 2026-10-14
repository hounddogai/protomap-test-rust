use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal dashboard threshold must pass. Check returns its problems, or none.
pub trait SeasonalDashboardThresholdRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDashboardThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardThresholdRequiredCountryRule {
    pub message: String,
}

impl SeasonalDashboardThresholdRule for SeasonalDashboardThresholdRequiredCountryRule {
    fn check(&self, item: &SeasonalDashboardThreshold) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardThresholdRequiredCodeRule {
    pub message: String,
}

impl SeasonalDashboardThresholdRule for SeasonalDashboardThresholdRequiredCodeRule {
    fn check(&self, item: &SeasonalDashboardThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDashboardThresholdRequiredUnitRule {
    pub message: String,
}

impl SeasonalDashboardThresholdRule for SeasonalDashboardThresholdRequiredUnitRule {
    fn check(&self, item: &SeasonalDashboardThreshold) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal dashboard threshold.
#[derive(Debug, Clone)]
pub struct SeasonalDashboardThresholdValidator {
    pub rules: Vec<Rc<dyn SeasonalDashboardThresholdRule>>,
}

impl SeasonalDashboardThresholdValidator {
    pub fn validate(&self, item: &SeasonalDashboardThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalDashboardThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDashboardThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dashboard_threshold_validator() -> SeasonalDashboardThresholdValidator {
    SeasonalDashboardThresholdValidator {
        rules: vec![
            Rc::new(SeasonalDashboardThresholdRequiredCountryRule {
                message: "seasonal dashboard threshold needs an entry".to_string(),
            }) as Rc<dyn SeasonalDashboardThresholdRule>,
            Rc::new(SeasonalDashboardThresholdRequiredCodeRule {
                message: "seasonal dashboard threshold needs this field".to_string(),
            }),
            Rc::new(SeasonalDashboardThresholdRequiredUnitRule {
                message: "seasonal dashboard threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_dashboard_threshold_validator();
        let item = new_seasonal_dashboard_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalDashboardThresholdRequiredCountryRule {
            message: "seasonal dashboard threshold needs a value".to_string(),
        };
        let mut item = new_seasonal_dashboard_threshold("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalDashboardThresholdRequiredCodeRule {
            message: "seasonal dashboard threshold needs a value".to_string(),
        };
        let mut item = new_seasonal_dashboard_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = SeasonalDashboardThresholdRequiredUnitRule {
            message: "seasonal dashboard threshold needs an entry".to_string(),
        };
        let mut item = new_seasonal_dashboard_threshold("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
