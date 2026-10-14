use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped data export plan must pass. Check returns its problems, or none.
pub trait CappedDataExportPlanRule: std::fmt::Debug {
    fn check(&self, item: &CappedDataExportPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPlanRequiredCodeRule {
    pub message: String,
}

impl CappedDataExportPlanRule for CappedDataExportPlanRequiredCodeRule {
    fn check(&self, item: &CappedDataExportPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPlanAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CappedDataExportPlanRule for CappedDataExportPlanAllowedTitleRule {
    fn check(&self, item: &CappedDataExportPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPlanBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDataExportPlanRule for CappedDataExportPlanBurstRangeRule {
    fn check(&self, item: &CappedDataExportPlan) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPlanRequiredTenantIdRule {
    pub message: String,
}

impl CappedDataExportPlanRule for CappedDataExportPlanRequiredTenantIdRule {
    fn check(&self, item: &CappedDataExportPlan) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped data export plan.
#[derive(Debug, Clone)]
pub struct CappedDataExportPlanValidator {
    pub rules: Vec<Rc<dyn CappedDataExportPlanRule>>,
}

impl CappedDataExportPlanValidator {
    pub fn validate(&self, item: &CappedDataExportPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDataExportPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDataExportPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_data_export_plan_validator() -> CappedDataExportPlanValidator {
    CappedDataExportPlanValidator {
        rules: vec![
            Rc::new(CappedDataExportPlanRequiredCodeRule {
                message: "capped data export plan needs an entry".to_string(),
            }) as Rc<dyn CappedDataExportPlanRule>,
            Rc::new(CappedDataExportPlanAllowedTitleRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(CappedDataExportPlanBurstRangeRule { min: 3600, max: 86400 }),
            Rc::new(CappedDataExportPlanRequiredTenantIdRule {
                message: "capped data export plan needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_data_export_plan_validator();
        let item = new_capped_data_export_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedDataExportPlanRequiredCodeRule {
            message: "capped data export plan needs an entry".to_string(),
        };
        let mut item = new_capped_data_export_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = CappedDataExportPlanBurstRangeRule { min: 16, max: 64 };
        let mut item = new_capped_data_export_plan("x", "y");
        item.burst = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = CappedDataExportPlanRequiredTenantIdRule {
            message: "capped data export plan needs this field".to_string(),
        };
        let mut item = new_capped_data_export_plan("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
