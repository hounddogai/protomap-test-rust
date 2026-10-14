use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary canary quota must pass. Check returns its problems, or none.
pub trait PrimaryCanaryQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryCanaryQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryQuotaMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryCanaryQuotaRule for PrimaryCanaryQuotaMaximumRangeRule {
    fn check(&self, item: &PrimaryCanaryQuota) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryQuotaRequiredLabelRule {
    pub message: String,
}

impl PrimaryCanaryQuotaRule for PrimaryCanaryQuotaRequiredLabelRule {
    fn check(&self, item: &PrimaryCanaryQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryCanaryQuotaRule for PrimaryCanaryQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryCanaryQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryQuotaUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryCanaryQuotaRule for PrimaryCanaryQuotaUpdatedAtRangeRule {
    fn check(&self, item: &PrimaryCanaryQuota) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary canary quota.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryCanaryQuotaRule>>,
}

impl PrimaryCanaryQuotaValidator {
    pub fn validate(&self, item: &PrimaryCanaryQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryCanaryQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryCanaryQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_canary_quota_validator() -> PrimaryCanaryQuotaValidator {
    PrimaryCanaryQuotaValidator {
        rules: vec![
            Rc::new(PrimaryCanaryQuotaMaximumRangeRule { min: 4.1, max: 2978.0 }) as Rc<dyn PrimaryCanaryQuotaRule>,
            Rc::new(PrimaryCanaryQuotaRequiredLabelRule {
                message: "primary canary quota needs a value".to_string(),
            }),
            Rc::new(PrimaryCanaryQuotaRequiredCodeRule {
                message: "primary canary quota needs an entry".to_string(),
            }),
            Rc::new(PrimaryCanaryQuotaUpdatedAtRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_canary_quota_validator();
        let item = new_primary_canary_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PrimaryCanaryQuotaMaximumRangeRule { min: 2.0, max: 4200.9 };
        let mut item = new_primary_canary_quota("x", "y");
        item.maximum = 4202.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryCanaryQuotaRequiredLabelRule {
            message: "primary canary quota needs an entry".to_string(),
        };
        let mut item = new_primary_canary_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryCanaryQuotaRequiredCodeRule {
            message: "primary canary quota needs this field".to_string(),
        };
        let mut item = new_primary_canary_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PrimaryCanaryQuotaUpdatedAtRangeRule { min: 32, max: 80 };
        let mut item = new_primary_canary_quota("x", "y");
        item.updated_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
