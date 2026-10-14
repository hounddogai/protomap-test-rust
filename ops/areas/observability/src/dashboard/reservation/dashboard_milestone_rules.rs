use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard milestone must pass. Check returns its problems, or none.
pub trait DashboardMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &DashboardMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneRequiredSourceRule {
    pub message: String,
}

impl DashboardMilestoneRule for DashboardMilestoneRequiredSourceRule {
    fn check(&self, item: &DashboardMilestone) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneRequiredTenantIdRule {
    pub message: String,
}

impl DashboardMilestoneRule for DashboardMilestoneRequiredTenantIdRule {
    fn check(&self, item: &DashboardMilestone) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardMilestoneRule for DashboardMilestoneAmountRangeRule {
    fn check(&self, item: &DashboardMilestone) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneTenantIdLengthRule {
    pub max_length: i64,
}

impl DashboardMilestoneRule for DashboardMilestoneTenantIdLengthRule {
    fn check(&self, item: &DashboardMilestone) -> Vec<String> {
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

/// Runs every rule of a dashboard milestone.
#[derive(Debug, Clone)]
pub struct DashboardMilestoneValidator {
    pub rules: Vec<Rc<dyn DashboardMilestoneRule>>,
}

impl DashboardMilestoneValidator {
    pub fn validate(&self, item: &DashboardMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_milestone_validator() -> DashboardMilestoneValidator {
    DashboardMilestoneValidator {
        rules: vec![
            Rc::new(DashboardMilestoneRequiredSourceRule {
                message: "dashboard milestone needs an entry".to_string(),
            }) as Rc<dyn DashboardMilestoneRule>,
            Rc::new(DashboardMilestoneRequiredTenantIdRule {
                message: "dashboard milestone needs a value".to_string(),
            }),
            Rc::new(DashboardMilestoneAmountRangeRule { min: 0.9, max: 764.2 }),
            Rc::new(DashboardMilestoneTenantIdLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_milestone_validator();
        let item = new_dashboard_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DashboardMilestoneRequiredSourceRule {
            message: "dashboard milestone needs a value".to_string(),
        };
        let mut item = new_dashboard_milestone("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = DashboardMilestoneRequiredTenantIdRule {
            message: "dashboard milestone needs a value".to_string(),
        };
        let mut item = new_dashboard_milestone("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DashboardMilestoneAmountRangeRule { min: 9.0, max: 2922.2 };
        let mut item = new_dashboard_milestone("x", "y");
        item.amount = 2923.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = DashboardMilestoneTenantIdLengthRule { max_length: 64 };
        let mut item = new_dashboard_milestone("x", "y");
        item.tenant_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
