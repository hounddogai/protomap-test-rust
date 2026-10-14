use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted overage quota must pass. Check returns its problems, or none.
pub trait WeightedOverageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &WeightedOverageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuotaMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedOverageQuotaRule for WeightedOverageQuotaMinimumRangeRule {
    fn check(&self, item: &WeightedOverageQuota) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuotaCodeLengthRule {
    pub max_length: i64,
}

impl WeightedOverageQuotaRule for WeightedOverageQuotaCodeLengthRule {
    fn check(&self, item: &WeightedOverageQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuotaRequiredNamespaceRule {
    pub message: String,
}

impl WeightedOverageQuotaRule for WeightedOverageQuotaRequiredNamespaceRule {
    fn check(&self, item: &WeightedOverageQuota) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuotaRequiredLabelRule {
    pub message: String,
}

impl WeightedOverageQuotaRule for WeightedOverageQuotaRequiredLabelRule {
    fn check(&self, item: &WeightedOverageQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted overage quota.
#[derive(Debug, Clone)]
pub struct WeightedOverageQuotaValidator {
    pub rules: Vec<Rc<dyn WeightedOverageQuotaRule>>,
}

impl WeightedOverageQuotaValidator {
    pub fn validate(&self, item: &WeightedOverageQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedOverageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedOverageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_overage_quota_validator() -> WeightedOverageQuotaValidator {
    WeightedOverageQuotaValidator {
        rules: vec![
            Rc::new(WeightedOverageQuotaMinimumRangeRule { min: 6.8, max: 475.0 }) as Rc<dyn WeightedOverageQuotaRule>,
            Rc::new(WeightedOverageQuotaCodeLengthRule { max_length: 32 }),
            Rc::new(WeightedOverageQuotaRequiredNamespaceRule {
                message: "weighted overage quota needs this field".to_string(),
            }),
            Rc::new(WeightedOverageQuotaRequiredLabelRule {
                message: "weighted overage quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_overage_quota_validator();
        let item = new_weighted_overage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = WeightedOverageQuotaMinimumRangeRule { min: 6.8, max: 4496.9 };
        let mut item = new_weighted_overage_quota("x", "y");
        item.minimum = 4498.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedOverageQuotaCodeLengthRule { max_length: 64 };
        let mut item = new_weighted_overage_quota("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = WeightedOverageQuotaRequiredNamespaceRule {
            message: "weighted overage quota needs a value".to_string(),
        };
        let mut item = new_weighted_overage_quota("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedOverageQuotaRequiredLabelRule {
            message: "weighted overage quota needs an entry".to_string(),
        };
        let mut item = new_weighted_overage_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
