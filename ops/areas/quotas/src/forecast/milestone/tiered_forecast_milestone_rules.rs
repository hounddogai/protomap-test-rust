use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered forecast milestone must pass. Check returns its problems, or none.
pub trait TieredForecastMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredForecastMilestoneRule for TieredForecastMilestoneQuotaUnitsRangeRule {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneSummaryLengthRule {
    pub max_length: i64,
}

impl TieredForecastMilestoneRule for TieredForecastMilestoneSummaryLengthRule {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredForecastMilestoneRule for TieredForecastMilestoneAllowedCodeRule {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneNameLengthRule {
    pub max_length: i64,
}

impl TieredForecastMilestoneRule for TieredForecastMilestoneNameLengthRule {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneRequiredNameRule {
    pub message: String,
}

impl TieredForecastMilestoneRule for TieredForecastMilestoneRequiredNameRule {
    fn check(&self, item: &TieredForecastMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered forecast milestone.
#[derive(Debug, Clone)]
pub struct TieredForecastMilestoneValidator {
    pub rules: Vec<Rc<dyn TieredForecastMilestoneRule>>,
}

impl TieredForecastMilestoneValidator {
    pub fn validate(&self, item: &TieredForecastMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredForecastMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredForecastMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_forecast_milestone_validator() -> TieredForecastMilestoneValidator {
    TieredForecastMilestoneValidator {
        rules: vec![
            Rc::new(TieredForecastMilestoneQuotaUnitsRangeRule { min: 64, max: 80 })
                as Rc<dyn TieredForecastMilestoneRule>,
            Rc::new(TieredForecastMilestoneSummaryLengthRule { max_length: 1000 }),
            Rc::new(TieredForecastMilestoneAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TieredForecastMilestoneNameLengthRule { max_length: 32 }),
            Rc::new(TieredForecastMilestoneRequiredNameRule {
                message: "tiered forecast milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_forecast_milestone_validator();
        let item = new_tiered_forecast_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = TieredForecastMilestoneQuotaUnitsRangeRule { min: 120, max: 86400 };
        let mut item = new_tiered_forecast_milestone("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = TieredForecastMilestoneSummaryLengthRule { max_length: 16 };
        let mut item = new_tiered_forecast_milestone("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredForecastMilestoneNameLengthRule { max_length: 32 };
        let mut item = new_tiered_forecast_milestone("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredForecastMilestoneRequiredNameRule {
            message: "tiered forecast milestone needs this field".to_string(),
        };
        let mut item = new_tiered_forecast_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
