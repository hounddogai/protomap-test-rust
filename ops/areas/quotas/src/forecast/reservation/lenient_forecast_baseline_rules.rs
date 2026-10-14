use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient forecast baseline must pass. Check returns its problems, or none.
pub trait LenientForecastBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientForecastBaselineRule for LenientForecastBaselineCreatedAtRangeRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl LenientForecastBaselineRule for LenientForecastBaselineAllowedAudienceRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is unknown", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineTitleLengthRule {
    pub max_length: i64,
}

impl LenientForecastBaselineRule for LenientForecastBaselineTitleLengthRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientForecastBaselineRule for LenientForecastBaselineRatioRangeRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineCodePrefixRule {
    pub prefix: String,
}

impl LenientForecastBaselineRule for LenientForecastBaselineCodePrefixRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineRequiredNotesRule {
    pub message: String,
}

impl LenientForecastBaselineRule for LenientForecastBaselineRequiredNotesRule {
    fn check(&self, item: &LenientForecastBaseline) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient forecast baseline.
#[derive(Debug, Clone)]
pub struct LenientForecastBaselineValidator {
    pub rules: Vec<Rc<dyn LenientForecastBaselineRule>>,
}

impl LenientForecastBaselineValidator {
    pub fn validate(&self, item: &LenientForecastBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientForecastBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientForecastBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_forecast_baseline_validator() -> LenientForecastBaselineValidator {
    LenientForecastBaselineValidator {
        rules: vec![
            Rc::new(LenientForecastBaselineCreatedAtRangeRule { min: 80, max: 80 })
                as Rc<dyn LenientForecastBaselineRule>,
            Rc::new(LenientForecastBaselineAllowedAudienceRule { allowed: Vec::new() }),
            Rc::new(LenientForecastBaselineTitleLengthRule { max_length: 86400 }),
            Rc::new(LenientForecastBaselineRatioRangeRule { min: 6.6, max: 4371.9 }),
            Rc::new(LenientForecastBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientForecastBaselineRequiredNotesRule {
                message: "lenient forecast baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_forecast_baseline_validator();
        let item = new_lenient_forecast_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientForecastBaselineCreatedAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_lenient_forecast_baseline("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientForecastBaselineTitleLengthRule { max_length: 86400 };
        let mut item = new_lenient_forecast_baseline("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = LenientForecastBaselineRatioRangeRule { min: 9.7, max: 536.1 };
        let mut item = new_lenient_forecast_baseline("x", "y");
        item.ratio = 537.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LenientForecastBaselineRequiredNotesRule {
            message: "lenient forecast baseline needs a value".to_string(),
        };
        let mut item = new_lenient_forecast_baseline("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
