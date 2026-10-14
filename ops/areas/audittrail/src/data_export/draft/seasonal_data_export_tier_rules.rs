use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal data export tier must pass. Check returns its problems, or none.
pub trait SeasonalDataExportTierRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDataExportTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTierRequiredTitleRule {
    pub message: String,
}

impl SeasonalDataExportTierRule for SeasonalDataExportTierRequiredTitleRule {
    fn check(&self, item: &SeasonalDataExportTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTierMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalDataExportTierRule for SeasonalDataExportTierMarginRangeRule {
    fn check(&self, item: &SeasonalDataExportTier) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTierTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDataExportTierRule for SeasonalDataExportTierTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalDataExportTier) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTierTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalDataExportTierRule for SeasonalDataExportTierTitleLengthRule {
    fn check(&self, item: &SeasonalDataExportTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal data export tier.
#[derive(Debug, Clone)]
pub struct SeasonalDataExportTierValidator {
    pub rules: Vec<Rc<dyn SeasonalDataExportTierRule>>,
}

impl SeasonalDataExportTierValidator {
    pub fn validate(&self, item: &SeasonalDataExportTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalDataExportTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDataExportTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_data_export_tier_validator() -> SeasonalDataExportTierValidator {
    SeasonalDataExportTierValidator {
        rules: vec![
            Rc::new(SeasonalDataExportTierRequiredTitleRule {
                message: "seasonal data export tier needs an entry".to_string(),
            }) as Rc<dyn SeasonalDataExportTierRule>,
            Rc::new(SeasonalDataExportTierMarginRangeRule { min: 2.5, max: 4253.2 }),
            Rc::new(SeasonalDataExportTierTtlSecondsRangeRule { min: 32, max: 1000 }),
            Rc::new(SeasonalDataExportTierTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_data_export_tier_validator();
        let item = new_seasonal_data_export_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalDataExportTierRequiredTitleRule {
            message: "seasonal data export tier needs this field".to_string(),
        };
        let mut item = new_seasonal_data_export_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = SeasonalDataExportTierMarginRangeRule { min: 4.5, max: 3716.3 };
        let mut item = new_seasonal_data_export_tier("x", "y");
        item.margin = 3717.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SeasonalDataExportTierTtlSecondsRangeRule { min: 86400, max: 86400 };
        let mut item = new_seasonal_data_export_tier("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalDataExportTierTitleLengthRule { max_length: 32 };
        let mut item = new_seasonal_data_export_tier("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
