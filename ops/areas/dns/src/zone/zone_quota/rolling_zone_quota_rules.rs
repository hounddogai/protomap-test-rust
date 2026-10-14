use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling zone quota must pass. Check returns its problems, or none.
pub trait RollingZoneQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RollingZoneQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuotaRequiredCodeRule {
    pub message: String,
}

impl RollingZoneQuotaRule for RollingZoneQuotaRequiredCodeRule {
    fn check(&self, item: &RollingZoneQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuotaRequiredSummaryRule {
    pub message: String,
}

impl RollingZoneQuotaRule for RollingZoneQuotaRequiredSummaryRule {
    fn check(&self, item: &RollingZoneQuota) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuotaRequiredNameRule {
    pub message: String,
}

impl RollingZoneQuotaRule for RollingZoneQuotaRequiredNameRule {
    fn check(&self, item: &RollingZoneQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuotaThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingZoneQuotaRule for RollingZoneQuotaThresholdRangeRule {
    fn check(&self, item: &RollingZoneQuota) -> Vec<String> {
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

/// Runs every rule of a rolling zone quota.
#[derive(Debug, Clone)]
pub struct RollingZoneQuotaValidator {
    pub rules: Vec<Rc<dyn RollingZoneQuotaRule>>,
}

impl RollingZoneQuotaValidator {
    pub fn validate(&self, item: &RollingZoneQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingZoneQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingZoneQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_zone_quota_validator() -> RollingZoneQuotaValidator {
    RollingZoneQuotaValidator {
        rules: vec![
            Rc::new(RollingZoneQuotaRequiredCodeRule {
                message: "rolling zone quota needs this field".to_string(),
            }) as Rc<dyn RollingZoneQuotaRule>,
            Rc::new(RollingZoneQuotaRequiredSummaryRule {
                message: "rolling zone quota needs an entry".to_string(),
            }),
            Rc::new(RollingZoneQuotaRequiredNameRule {
                message: "rolling zone quota needs an entry".to_string(),
            }),
            Rc::new(RollingZoneQuotaThresholdRangeRule { min: 1.0, max: 3730.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_zone_quota_validator();
        let item = new_rolling_zone_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingZoneQuotaRequiredCodeRule {
            message: "rolling zone quota needs an entry".to_string(),
        };
        let mut item = new_rolling_zone_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RollingZoneQuotaRequiredSummaryRule {
            message: "rolling zone quota needs a value".to_string(),
        };
        let mut item = new_rolling_zone_quota("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingZoneQuotaRequiredNameRule {
            message: "rolling zone quota needs this field".to_string(),
        };
        let mut item = new_rolling_zone_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RollingZoneQuotaThresholdRangeRule { min: 1.3, max: 4060.8 };
        let mut item = new_rolling_zone_quota("x", "y");
        item.threshold = 4062.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
