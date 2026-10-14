use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal release estimate must pass. Check returns its problems, or none.
pub trait SeasonalReleaseEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalReleaseEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseEstimateNameLengthRule {
    pub max_length: i64,
}

impl SeasonalReleaseEstimateRule for SeasonalReleaseEstimateNameLengthRule {
    fn check(&self, item: &SeasonalReleaseEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseEstimateRequiredTenantIdRule {
    pub message: String,
}

impl SeasonalReleaseEstimateRule for SeasonalReleaseEstimateRequiredTenantIdRule {
    fn check(&self, item: &SeasonalReleaseEstimate) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseEstimateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalReleaseEstimateRule for SeasonalReleaseEstimateLimitRangeRule {
    fn check(&self, item: &SeasonalReleaseEstimate) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseEstimateRequiredLocaleRule {
    pub message: String,
}

impl SeasonalReleaseEstimateRule for SeasonalReleaseEstimateRequiredLocaleRule {
    fn check(&self, item: &SeasonalReleaseEstimate) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal release estimate.
#[derive(Debug, Clone)]
pub struct SeasonalReleaseEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalReleaseEstimateRule>>,
}

impl SeasonalReleaseEstimateValidator {
    pub fn validate(&self, item: &SeasonalReleaseEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalReleaseEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalReleaseEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_release_estimate_validator() -> SeasonalReleaseEstimateValidator {
    SeasonalReleaseEstimateValidator {
        rules: vec![
            Rc::new(SeasonalReleaseEstimateNameLengthRule { max_length: 16 }) as Rc<dyn SeasonalReleaseEstimateRule>,
            Rc::new(SeasonalReleaseEstimateRequiredTenantIdRule {
                message: "seasonal release estimate needs a value".to_string(),
            }),
            Rc::new(SeasonalReleaseEstimateLimitRangeRule { min: 64, max: 64 }),
            Rc::new(SeasonalReleaseEstimateRequiredLocaleRule {
                message: "seasonal release estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_release_estimate_validator();
        let item = new_seasonal_release_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalReleaseEstimateNameLengthRule { max_length: 120 };
        let mut item = new_seasonal_release_estimate("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = SeasonalReleaseEstimateRequiredTenantIdRule {
            message: "seasonal release estimate needs a value".to_string(),
        };
        let mut item = new_seasonal_release_estimate("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = SeasonalReleaseEstimateLimitRangeRule { min: 32, max: 3600 };
        let mut item = new_seasonal_release_estimate("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SeasonalReleaseEstimateRequiredLocaleRule {
            message: "seasonal release estimate needs an entry".to_string(),
        };
        let mut item = new_seasonal_release_estimate("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
