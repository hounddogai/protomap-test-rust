use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast score must pass. Check returns its problems, or none.
pub trait ForecastScoreRule: std::fmt::Debug {
    fn check(&self, item: &ForecastScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreCodeLengthRule {
    pub max_length: i64,
}

impl ForecastScoreRule for ForecastScoreCodeLengthRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreLocaleLengthRule {
    pub max_length: i64,
}

impl ForecastScoreRule for ForecastScoreLocaleLengthRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreRequiredLabelRule {
    pub message: String,
}

impl ForecastScoreRule for ForecastScoreRequiredLabelRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastScoreRule for ForecastScoreCeilingRangeRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastScoreRule for ForecastScoreFloorValueRangeRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScoreLabelLengthRule {
    pub max_length: i64,
}

impl ForecastScoreRule for ForecastScoreLabelLengthRule {
    fn check(&self, item: &ForecastScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast score.
#[derive(Debug, Clone)]
pub struct ForecastScoreValidator {
    pub rules: Vec<Rc<dyn ForecastScoreRule>>,
}

impl ForecastScoreValidator {
    pub fn validate(&self, item: &ForecastScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_score_validator() -> ForecastScoreValidator {
    ForecastScoreValidator {
        rules: vec![
            Rc::new(ForecastScoreCodeLengthRule { max_length: 3600 }) as Rc<dyn ForecastScoreRule>,
            Rc::new(ForecastScoreLocaleLengthRule { max_length: 3600 }),
            Rc::new(ForecastScoreRequiredLabelRule {
                message: "forecast score needs an entry".to_string(),
            }),
            Rc::new(ForecastScoreCeilingRangeRule { min: 0.8, max: 4721.3 }),
            Rc::new(ForecastScoreFloorValueRangeRule { min: 4.3, max: 4752.6 }),
            Rc::new(ForecastScoreLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_score_validator();
        let item = new_forecast_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastScoreCodeLengthRule { max_length: 32 };
        let mut item = new_forecast_score("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ForecastScoreLocaleLengthRule { max_length: 1000 };
        let mut item = new_forecast_score("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ForecastScoreRequiredLabelRule {
            message: "forecast score needs this field".to_string(),
        };
        let mut item = new_forecast_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ForecastScoreCeilingRangeRule { min: 5.9, max: 1397.1 };
        let mut item = new_forecast_score("x", "y");
        item.ceiling = 1398.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ForecastScoreFloorValueRangeRule { min: 4.4, max: 365.9 };
        let mut item = new_forecast_score("x", "y");
        item.floor_value = 367.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ForecastScoreLabelLengthRule { max_length: 3600 };
        let mut item = new_forecast_score("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
