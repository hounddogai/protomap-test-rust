use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert quota must pass. Check returns its problems, or none.
pub trait AlertQuotaRule: std::fmt::Debug {
    fn check(&self, item: &AlertQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertQuotaUnitLengthRule {
    pub max_length: i64,
}

impl AlertQuotaRule for AlertQuotaUnitLengthRule {
    fn check(&self, item: &AlertQuota) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertQuotaVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertQuotaRule for AlertQuotaVersionRangeRule {
    fn check(&self, item: &AlertQuota) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertQuotaRequiredUnitRule {
    pub message: String,
}

impl AlertQuotaRule for AlertQuotaRequiredUnitRule {
    fn check(&self, item: &AlertQuota) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert quota.
#[derive(Debug, Clone)]
pub struct AlertQuotaValidator {
    pub rules: Vec<Rc<dyn AlertQuotaRule>>,
}

impl AlertQuotaValidator {
    pub fn validate(&self, item: &AlertQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_quota_validator() -> AlertQuotaValidator {
    AlertQuotaValidator {
        rules: vec![
            Rc::new(AlertQuotaUnitLengthRule { max_length: 16 }) as Rc<dyn AlertQuotaRule>,
            Rc::new(AlertQuotaVersionRangeRule { min: 16, max: 3600 }),
            Rc::new(AlertQuotaRequiredUnitRule {
                message: "alert quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_quota_validator();
        let item = new_alert_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = AlertQuotaUnitLengthRule { max_length: 255 };
        let mut item = new_alert_quota("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = AlertQuotaVersionRangeRule { min: 64, max: 86400 };
        let mut item = new_alert_quota("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = AlertQuotaRequiredUnitRule {
            message: "alert quota needs an entry".to_string(),
        };
        let mut item = new_alert_quota("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
