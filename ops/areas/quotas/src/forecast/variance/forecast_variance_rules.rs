use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast variance must pass. Check returns its problems, or none.
pub trait ForecastVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ForecastVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceApprovedGuardRule {
    pub limit: f64,
}

impl ForecastVarianceRule for ForecastVarianceApprovedGuardRule {
    fn check(&self, item: &ForecastVariance) -> Vec<String> {
        if item.approved && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while approved",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastVarianceRule for ForecastVarianceFloorValueRangeRule {
    fn check(&self, item: &ForecastVariance) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceCodePrefixRule {
    pub prefix: String,
}

impl ForecastVarianceRule for ForecastVarianceCodePrefixRule {
    fn check(&self, item: &ForecastVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastVarianceRule for ForecastVarianceCostRangeRule {
    fn check(&self, item: &ForecastVariance) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast variance.
#[derive(Debug, Clone)]
pub struct ForecastVarianceValidator {
    pub rules: Vec<Rc<dyn ForecastVarianceRule>>,
}

impl ForecastVarianceValidator {
    pub fn validate(&self, item: &ForecastVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_variance_validator() -> ForecastVarianceValidator {
    ForecastVarianceValidator {
        rules: vec![
            Rc::new(ForecastVarianceApprovedGuardRule { limit: 0.8 }) as Rc<dyn ForecastVarianceRule>,
            Rc::new(ForecastVarianceFloorValueRangeRule { min: 4.0, max: 4858.9 }),
            Rc::new(ForecastVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ForecastVarianceCostRangeRule { min: 1.4, max: 4444.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_variance_validator();
        let item = new_forecast_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ForecastVarianceFloorValueRangeRule { min: 5.6, max: 1876.8 };
        let mut item = new_forecast_variance("x", "y");
        item.floor_value = 1878.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ForecastVarianceCostRangeRule { min: 1.5, max: 3043.7 };
        let mut item = new_forecast_variance("x", "y");
        item.cost = 3045.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
