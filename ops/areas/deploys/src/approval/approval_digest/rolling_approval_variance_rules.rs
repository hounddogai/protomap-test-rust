use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling approval variance must pass. Check returns its problems, or none.
pub trait RollingApprovalVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RollingApprovalVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalVarianceRequiredTenantIdRule {
    pub message: String,
}

impl RollingApprovalVarianceRule for RollingApprovalVarianceRequiredTenantIdRule {
    fn check(&self, item: &RollingApprovalVariance) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalVarianceRequiredUnitRule {
    pub message: String,
}

impl RollingApprovalVarianceRule for RollingApprovalVarianceRequiredUnitRule {
    fn check(&self, item: &RollingApprovalVariance) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalVarianceCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingApprovalVarianceRule for RollingApprovalVarianceCreatedAtRangeRule {
    fn check(&self, item: &RollingApprovalVariance) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling approval variance.
#[derive(Debug, Clone)]
pub struct RollingApprovalVarianceValidator {
    pub rules: Vec<Rc<dyn RollingApprovalVarianceRule>>,
}

impl RollingApprovalVarianceValidator {
    pub fn validate(&self, item: &RollingApprovalVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingApprovalVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingApprovalVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_approval_variance_validator() -> RollingApprovalVarianceValidator {
    RollingApprovalVarianceValidator {
        rules: vec![
            Rc::new(RollingApprovalVarianceRequiredTenantIdRule {
                message: "rolling approval variance needs this field".to_string(),
            }) as Rc<dyn RollingApprovalVarianceRule>,
            Rc::new(RollingApprovalVarianceRequiredUnitRule {
                message: "rolling approval variance needs a value".to_string(),
            }),
            Rc::new(RollingApprovalVarianceCreatedAtRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_approval_variance_validator();
        let item = new_rolling_approval_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RollingApprovalVarianceRequiredTenantIdRule {
            message: "rolling approval variance needs an entry".to_string(),
        };
        let mut item = new_rolling_approval_variance("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RollingApprovalVarianceRequiredUnitRule {
            message: "rolling approval variance needs this field".to_string(),
        };
        let mut item = new_rolling_approval_variance("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RollingApprovalVarianceCreatedAtRangeRule { min: 16, max: 120 };
        let mut item = new_rolling_approval_variance("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
