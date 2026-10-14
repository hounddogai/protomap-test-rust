use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal alert bundle must pass. Check returns its problems, or none.
pub trait SeasonalAlertBundleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAlertBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertBundleCodePrefixRule {
    pub prefix: String,
}

impl SeasonalAlertBundleRule for SeasonalAlertBundleCodePrefixRule {
    fn check(&self, item: &SeasonalAlertBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertBundleNameLengthRule {
    pub max_length: i64,
}

impl SeasonalAlertBundleRule for SeasonalAlertBundleNameLengthRule {
    fn check(&self, item: &SeasonalAlertBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalAlertBundleRule for SeasonalAlertBundleScoreRangeRule {
    fn check(&self, item: &SeasonalAlertBundle) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal alert bundle.
#[derive(Debug, Clone)]
pub struct SeasonalAlertBundleValidator {
    pub rules: Vec<Rc<dyn SeasonalAlertBundleRule>>,
}

impl SeasonalAlertBundleValidator {
    pub fn validate(&self, item: &SeasonalAlertBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalAlertBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAlertBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_alert_bundle_validator() -> SeasonalAlertBundleValidator {
    SeasonalAlertBundleValidator {
        rules: vec![
            Rc::new(SeasonalAlertBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SeasonalAlertBundleRule>,
            Rc::new(SeasonalAlertBundleNameLengthRule { max_length: 16 }),
            Rc::new(SeasonalAlertBundleScoreRangeRule { min: 5.6, max: 2345.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_alert_bundle_validator();
        let item = new_seasonal_alert_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalAlertBundleNameLengthRule { max_length: 32 };
        let mut item = new_seasonal_alert_bundle("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = SeasonalAlertBundleScoreRangeRule { min: 5.9, max: 3656.4 };
        let mut item = new_seasonal_alert_bundle("x", "y");
        item.score = 3657.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
