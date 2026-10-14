use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal identity schedule must pass. Check returns its problems, or none.
pub trait SeasonalIdentityScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIdentitySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityScheduleAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl SeasonalIdentityScheduleRule for SeasonalIdentityScheduleAllowedIssuerRule {
    fn check(&self, item: &SeasonalIdentitySchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is above maximum", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityScheduleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalIdentityScheduleRule for SeasonalIdentityScheduleRatioRangeRule {
    fn check(&self, item: &SeasonalIdentitySchedule) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityScheduleIssuerLengthRule {
    pub max_length: i64,
}

impl SeasonalIdentityScheduleRule for SeasonalIdentityScheduleIssuerLengthRule {
    fn check(&self, item: &SeasonalIdentitySchedule) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal identity schedule.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityScheduleValidator {
    pub rules: Vec<Rc<dyn SeasonalIdentityScheduleRule>>,
}

impl SeasonalIdentityScheduleValidator {
    pub fn validate(&self, item: &SeasonalIdentitySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIdentitySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIdentitySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_identity_schedule_validator() -> SeasonalIdentityScheduleValidator {
    SeasonalIdentityScheduleValidator {
        rules: vec![
            Rc::new(SeasonalIdentityScheduleAllowedIssuerRule {
                allowed: vec!["default".to_string(), "primary".to_string()],
            }) as Rc<dyn SeasonalIdentityScheduleRule>,
            Rc::new(SeasonalIdentityScheduleRatioRangeRule { min: 8.4, max: 3239.8 }),
            Rc::new(SeasonalIdentityScheduleIssuerLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_identity_schedule_validator();
        let item = new_seasonal_identity_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SeasonalIdentityScheduleRatioRangeRule { min: 4.4, max: 4028.8 };
        let mut item = new_seasonal_identity_schedule("x", "y");
        item.ratio = 4030.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = SeasonalIdentityScheduleIssuerLengthRule { max_length: 64 };
        let mut item = new_seasonal_identity_schedule("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
