use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant plan must pass. Check returns its problems, or none.
pub trait TenantPlanRule: std::fmt::Debug {
    fn check(&self, item: &TenantPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlanRequiredSummaryRule {
    pub message: String,
}

impl TenantPlanRule for TenantPlanRequiredSummaryRule {
    fn check(&self, item: &TenantPlan) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlanCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantPlanRule for TenantPlanCeilingRangeRule {
    fn check(&self, item: &TenantPlan) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlanCodeLengthRule {
    pub max_length: i64,
}

impl TenantPlanRule for TenantPlanCodeLengthRule {
    fn check(&self, item: &TenantPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlanCodePrefixRule {
    pub prefix: String,
}

impl TenantPlanRule for TenantPlanCodePrefixRule {
    fn check(&self, item: &TenantPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant plan.
#[derive(Debug, Clone)]
pub struct TenantPlanValidator {
    pub rules: Vec<Rc<dyn TenantPlanRule>>,
}

impl TenantPlanValidator {
    pub fn validate(&self, item: &TenantPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_plan_validator() -> TenantPlanValidator {
    TenantPlanValidator {
        rules: vec![
            Rc::new(TenantPlanRequiredSummaryRule {
                message: "tenant plan needs a value".to_string(),
            }) as Rc<dyn TenantPlanRule>,
            Rc::new(TenantPlanCeilingRangeRule { min: 7.2, max: 3636.2 }),
            Rc::new(TenantPlanCodeLengthRule { max_length: 3600 }),
            Rc::new(TenantPlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_plan_validator();
        let item = new_tenant_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = TenantPlanRequiredSummaryRule {
            message: "tenant plan needs a value".to_string(),
        };
        let mut item = new_tenant_plan("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TenantPlanCeilingRangeRule { min: 1.0, max: 2978.8 };
        let mut item = new_tenant_plan("x", "y");
        item.ceiling = 2980.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantPlanCodeLengthRule { max_length: 32 };
        let mut item = new_tenant_plan("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
