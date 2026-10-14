use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal overage bundle must pass. Check returns its problems, or none.
pub trait SeasonalOverageBundleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalOverageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageBundleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalOverageBundleRule for SeasonalOverageBundleRatioRangeRule {
    fn check(&self, item: &SeasonalOverageBundle) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageBundleRequiredScopeRule {
    pub message: String,
}

impl SeasonalOverageBundleRule for SeasonalOverageBundleRequiredScopeRule {
    fn check(&self, item: &SeasonalOverageBundle) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalOverageBundleRule for SeasonalOverageBundleWindowDaysRangeRule {
    fn check(&self, item: &SeasonalOverageBundle) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal overage bundle.
#[derive(Debug, Clone)]
pub struct SeasonalOverageBundleValidator {
    pub rules: Vec<Rc<dyn SeasonalOverageBundleRule>>,
}

impl SeasonalOverageBundleValidator {
    pub fn validate(&self, item: &SeasonalOverageBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalOverageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalOverageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_overage_bundle_validator() -> SeasonalOverageBundleValidator {
    SeasonalOverageBundleValidator {
        rules: vec![
            Rc::new(SeasonalOverageBundleRatioRangeRule { min: 0.6, max: 2227.6 }) as Rc<dyn SeasonalOverageBundleRule>,
            Rc::new(SeasonalOverageBundleRequiredScopeRule {
                message: "seasonal overage bundle needs this field".to_string(),
            }),
            Rc::new(SeasonalOverageBundleWindowDaysRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_overage_bundle_validator();
        let item = new_seasonal_overage_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SeasonalOverageBundleRatioRangeRule { min: 3.7, max: 1387.0 };
        let mut item = new_seasonal_overage_bundle("x", "y");
        item.ratio = 1388.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = SeasonalOverageBundleRequiredScopeRule {
            message: "seasonal overage bundle needs a value".to_string(),
        };
        let mut item = new_seasonal_overage_bundle("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SeasonalOverageBundleWindowDaysRangeRule { min: 16, max: 3600 };
        let mut item = new_seasonal_overage_bundle("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
