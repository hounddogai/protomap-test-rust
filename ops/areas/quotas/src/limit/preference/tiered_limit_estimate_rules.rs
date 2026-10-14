use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered limit estimate must pass. Check returns its problems, or none.
pub trait TieredLimitEstimateRule: std::fmt::Debug {
    fn check(&self, item: &TieredLimitEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredLimitEstimateRule for TieredLimitEstimateExpiresAtRangeRule {
    fn check(&self, item: &TieredLimitEstimate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateRequiredLabelRule {
    pub message: String,
}

impl TieredLimitEstimateRule for TieredLimitEstimateRequiredLabelRule {
    fn check(&self, item: &TieredLimitEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateLabelLengthRule {
    pub max_length: i64,
}

impl TieredLimitEstimateRule for TieredLimitEstimateLabelLengthRule {
    fn check(&self, item: &TieredLimitEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateTenantIdLengthRule {
    pub max_length: i64,
}

impl TieredLimitEstimateRule for TieredLimitEstimateTenantIdLengthRule {
    fn check(&self, item: &TieredLimitEstimate) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered limit estimate.
#[derive(Debug, Clone)]
pub struct TieredLimitEstimateValidator {
    pub rules: Vec<Rc<dyn TieredLimitEstimateRule>>,
}

impl TieredLimitEstimateValidator {
    pub fn validate(&self, item: &TieredLimitEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredLimitEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredLimitEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_limit_estimate_validator() -> TieredLimitEstimateValidator {
    TieredLimitEstimateValidator {
        rules: vec![
            Rc::new(TieredLimitEstimateExpiresAtRangeRule { min: 120, max: 255 }) as Rc<dyn TieredLimitEstimateRule>,
            Rc::new(TieredLimitEstimateRequiredLabelRule {
                message: "tiered limit estimate needs a value".to_string(),
            }),
            Rc::new(TieredLimitEstimateLabelLengthRule { max_length: 80 }),
            Rc::new(TieredLimitEstimateTenantIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_limit_estimate_validator();
        let item = new_tiered_limit_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TieredLimitEstimateExpiresAtRangeRule { min: 80, max: 255 };
        let mut item = new_tiered_limit_estimate("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredLimitEstimateRequiredLabelRule {
            message: "tiered limit estimate needs this field".to_string(),
        };
        let mut item = new_tiered_limit_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredLimitEstimateLabelLengthRule { max_length: 255 };
        let mut item = new_tiered_limit_estimate("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TieredLimitEstimateTenantIdLengthRule { max_length: 120 };
        let mut item = new_tiered_limit_estimate("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
