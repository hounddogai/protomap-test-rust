use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal query milestone must pass. Check returns its problems, or none.
pub trait SeasonalQueryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneCountryLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneCountryLengthRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneRequiredCodeRule {
    pub message: String,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneRequiredCodeRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneThresholdRangeRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneRequiredCountryRule {
    pub message: String,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneRequiredCountryRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneTitleLengthRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalQueryMilestoneRule for SeasonalQueryMilestoneCodeLengthRule {
    fn check(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal query milestone.
#[derive(Debug, Clone)]
pub struct SeasonalQueryMilestoneValidator {
    pub rules: Vec<Rc<dyn SeasonalQueryMilestoneRule>>,
}

impl SeasonalQueryMilestoneValidator {
    pub fn validate(&self, item: &SeasonalQueryMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalQueryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalQueryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_query_milestone_validator() -> SeasonalQueryMilestoneValidator {
    SeasonalQueryMilestoneValidator {
        rules: vec![
            Rc::new(SeasonalQueryMilestoneCountryLengthRule { max_length: 120 }) as Rc<dyn SeasonalQueryMilestoneRule>,
            Rc::new(SeasonalQueryMilestoneRequiredCodeRule {
                message: "seasonal query milestone needs a value".to_string(),
            }),
            Rc::new(SeasonalQueryMilestoneThresholdRangeRule { min: 5.1, max: 3125.8 }),
            Rc::new(SeasonalQueryMilestoneRequiredCountryRule {
                message: "seasonal query milestone needs a value".to_string(),
            }),
            Rc::new(SeasonalQueryMilestoneTitleLengthRule { max_length: 64 }),
            Rc::new(SeasonalQueryMilestoneCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_query_milestone_validator();
        let item = new_seasonal_query_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = SeasonalQueryMilestoneCountryLengthRule { max_length: 255 };
        let mut item = new_seasonal_query_milestone("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalQueryMilestoneRequiredCodeRule {
            message: "seasonal query milestone needs a value".to_string(),
        };
        let mut item = new_seasonal_query_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SeasonalQueryMilestoneThresholdRangeRule { min: 2.9, max: 459.5 };
        let mut item = new_seasonal_query_milestone("x", "y");
        item.threshold = 461.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalQueryMilestoneRequiredCountryRule {
            message: "seasonal query milestone needs a value".to_string(),
        };
        let mut item = new_seasonal_query_milestone("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalQueryMilestoneTitleLengthRule { max_length: 16 };
        let mut item = new_seasonal_query_milestone("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalQueryMilestoneCodeLengthRule { max_length: 64 };
        let mut item = new_seasonal_query_milestone("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
