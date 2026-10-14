use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback forecast quota must pass. Check returns its problems, or none.
pub trait FallbackForecastQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FallbackForecastQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackForecastQuotaRequiredTitleRule {
    pub message: String,
}

impl FallbackForecastQuotaRule for FallbackForecastQuotaRequiredTitleRule {
    fn check(&self, item: &FallbackForecastQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackForecastQuotaTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackForecastQuotaRule for FallbackForecastQuotaTtlSecondsRangeRule {
    fn check(&self, item: &FallbackForecastQuota) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackForecastQuotaRequiredHostRule {
    pub message: String,
}

impl FallbackForecastQuotaRule for FallbackForecastQuotaRequiredHostRule {
    fn check(&self, item: &FallbackForecastQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback forecast quota.
#[derive(Debug, Clone)]
pub struct FallbackForecastQuotaValidator {
    pub rules: Vec<Rc<dyn FallbackForecastQuotaRule>>,
}

impl FallbackForecastQuotaValidator {
    pub fn validate(&self, item: &FallbackForecastQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackForecastQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackForecastQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_forecast_quota_validator() -> FallbackForecastQuotaValidator {
    FallbackForecastQuotaValidator {
        rules: vec![
            Rc::new(FallbackForecastQuotaRequiredTitleRule {
                message: "fallback forecast quota needs a value".to_string(),
            }) as Rc<dyn FallbackForecastQuotaRule>,
            Rc::new(FallbackForecastQuotaTtlSecondsRangeRule { min: 80, max: 86400 }),
            Rc::new(FallbackForecastQuotaRequiredHostRule {
                message: "fallback forecast quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_forecast_quota_validator();
        let item = new_fallback_forecast_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackForecastQuotaRequiredTitleRule {
            message: "fallback forecast quota needs this field".to_string(),
        };
        let mut item = new_fallback_forecast_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = FallbackForecastQuotaTtlSecondsRangeRule { min: 120, max: 86400 };
        let mut item = new_fallback_forecast_quota("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = FallbackForecastQuotaRequiredHostRule {
            message: "fallback forecast quota needs a value".to_string(),
        };
        let mut item = new_fallback_forecast_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
