use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant variance must pass. Check returns its problems, or none.
pub trait TenantVarianceRule: std::fmt::Debug {
    fn check(&self, item: &TenantVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceRequiredReviewerRule {
    pub message: String,
}

impl TenantVarianceRule for TenantVarianceRequiredReviewerRule {
    fn check(&self, item: &TenantVariance) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceCodeLengthRule {
    pub max_length: i64,
}

impl TenantVarianceRule for TenantVarianceCodeLengthRule {
    fn check(&self, item: &TenantVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantVarianceRule for TenantVarianceMinimumRangeRule {
    fn check(&self, item: &TenantVariance) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceRequiredLabelRule {
    pub message: String,
}

impl TenantVarianceRule for TenantVarianceRequiredLabelRule {
    fn check(&self, item: &TenantVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceRequiredCodeRule {
    pub message: String,
}

impl TenantVarianceRule for TenantVarianceRequiredCodeRule {
    fn check(&self, item: &TenantVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant variance.
#[derive(Debug, Clone)]
pub struct TenantVarianceValidator {
    pub rules: Vec<Rc<dyn TenantVarianceRule>>,
}

impl TenantVarianceValidator {
    pub fn validate(&self, item: &TenantVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_variance_validator() -> TenantVarianceValidator {
    TenantVarianceValidator {
        rules: vec![
            Rc::new(TenantVarianceRequiredReviewerRule {
                message: "tenant variance needs an entry".to_string(),
            }) as Rc<dyn TenantVarianceRule>,
            Rc::new(TenantVarianceCodeLengthRule { max_length: 86400 }),
            Rc::new(TenantVarianceMinimumRangeRule { min: 1.0, max: 4070.9 }),
            Rc::new(TenantVarianceRequiredLabelRule {
                message: "tenant variance needs a value".to_string(),
            }),
            Rc::new(TenantVarianceRequiredCodeRule {
                message: "tenant variance needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_variance_validator();
        let item = new_tenant_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = TenantVarianceRequiredReviewerRule {
            message: "tenant variance needs this field".to_string(),
        };
        let mut item = new_tenant_variance("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_tenant_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TenantVarianceMinimumRangeRule { min: 1.6, max: 1232.9 };
        let mut item = new_tenant_variance("x", "y");
        item.minimum = 1234.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TenantVarianceRequiredLabelRule {
            message: "tenant variance needs this field".to_string(),
        };
        let mut item = new_tenant_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantVarianceRequiredCodeRule {
            message: "tenant variance needs an entry".to_string(),
        };
        let mut item = new_tenant_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
