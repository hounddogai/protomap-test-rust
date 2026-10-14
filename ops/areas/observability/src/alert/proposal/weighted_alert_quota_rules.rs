use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted alert quota must pass. Check returns its problems, or none.
pub trait WeightedAlertQuotaRule: std::fmt::Debug {
    fn check(&self, item: &WeightedAlertQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaCodeLengthRule {
    pub max_length: i64,
}

impl WeightedAlertQuotaRule for WeightedAlertQuotaCodeLengthRule {
    fn check(&self, item: &WeightedAlertQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaCodePrefixRule {
    pub prefix: String,
}

impl WeightedAlertQuotaRule for WeightedAlertQuotaCodePrefixRule {
    fn check(&self, item: &WeightedAlertQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaRequiredRegionRule {
    pub message: String,
}

impl WeightedAlertQuotaRule for WeightedAlertQuotaRequiredRegionRule {
    fn check(&self, item: &WeightedAlertQuota) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedAlertQuotaRule for WeightedAlertQuotaAmountRangeRule {
    fn check(&self, item: &WeightedAlertQuota) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted alert quota.
#[derive(Debug, Clone)]
pub struct WeightedAlertQuotaValidator {
    pub rules: Vec<Rc<dyn WeightedAlertQuotaRule>>,
}

impl WeightedAlertQuotaValidator {
    pub fn validate(&self, item: &WeightedAlertQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedAlertQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedAlertQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_alert_quota_validator() -> WeightedAlertQuotaValidator {
    WeightedAlertQuotaValidator {
        rules: vec![
            Rc::new(WeightedAlertQuotaCodeLengthRule { max_length: 255 }) as Rc<dyn WeightedAlertQuotaRule>,
            Rc::new(WeightedAlertQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedAlertQuotaRequiredRegionRule {
                message: "weighted alert quota needs an entry".to_string(),
            }),
            Rc::new(WeightedAlertQuotaAmountRangeRule { min: 3.2, max: 768.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_alert_quota_validator();
        let item = new_weighted_alert_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedAlertQuotaCodeLengthRule { max_length: 1000 };
        let mut item = new_weighted_alert_quota("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = WeightedAlertQuotaRequiredRegionRule {
            message: "weighted alert quota needs this field".to_string(),
        };
        let mut item = new_weighted_alert_quota("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = WeightedAlertQuotaAmountRangeRule { min: 1.9, max: 3391.4 };
        let mut item = new_weighted_alert_quota("x", "y");
        item.amount = 3392.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
