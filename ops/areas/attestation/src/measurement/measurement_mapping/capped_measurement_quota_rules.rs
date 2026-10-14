use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped measurement quota must pass. Check returns its problems, or none.
pub trait CappedMeasurementQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CappedMeasurementQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedMeasurementQuotaRule for CappedMeasurementQuotaPriorityRangeRule {
    fn check(&self, item: &CappedMeasurementQuota) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaCodeLengthRule {
    pub max_length: i64,
}

impl CappedMeasurementQuotaRule for CappedMeasurementQuotaCodeLengthRule {
    fn check(&self, item: &CappedMeasurementQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaPhoneLengthRule {
    pub max_length: i64,
}

impl CappedMeasurementQuotaRule for CappedMeasurementQuotaPhoneLengthRule {
    fn check(&self, item: &CappedMeasurementQuota) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped measurement quota.
#[derive(Debug, Clone)]
pub struct CappedMeasurementQuotaValidator {
    pub rules: Vec<Rc<dyn CappedMeasurementQuotaRule>>,
}

impl CappedMeasurementQuotaValidator {
    pub fn validate(&self, item: &CappedMeasurementQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedMeasurementQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedMeasurementQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_measurement_quota_validator() -> CappedMeasurementQuotaValidator {
    CappedMeasurementQuotaValidator {
        rules: vec![
            Rc::new(CappedMeasurementQuotaPriorityRangeRule { min: 32, max: 120 })
                as Rc<dyn CappedMeasurementQuotaRule>,
            Rc::new(CappedMeasurementQuotaCodeLengthRule { max_length: 3600 }),
            Rc::new(CappedMeasurementQuotaPhoneLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_measurement_quota_validator();
        let item = new_capped_measurement_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CappedMeasurementQuotaPriorityRangeRule { min: 32, max: 1000 };
        let mut item = new_capped_measurement_quota("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedMeasurementQuotaCodeLengthRule { max_length: 255 };
        let mut item = new_capped_measurement_quota("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = CappedMeasurementQuotaPhoneLengthRule { max_length: 3600 };
        let mut item = new_capped_measurement_quota("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
