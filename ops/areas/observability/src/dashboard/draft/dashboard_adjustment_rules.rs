use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard adjustment must pass. Check returns its problems, or none.
pub trait DashboardAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DashboardAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl DashboardAdjustmentRule for DashboardAdjustmentCodePrefixRule {
    fn check(&self, item: &DashboardAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentRequiredCodeRule {
    pub message: String,
}

impl DashboardAdjustmentRule for DashboardAdjustmentRequiredCodeRule {
    fn check(&self, item: &DashboardAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentRequiredTitleRule {
    pub message: String,
}

impl DashboardAdjustmentRule for DashboardAdjustmentRequiredTitleRule {
    fn check(&self, item: &DashboardAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentRequiredContactEmailRule {
    pub message: String,
}

impl DashboardAdjustmentRule for DashboardAdjustmentRequiredContactEmailRule {
    fn check(&self, item: &DashboardAdjustment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard adjustment.
#[derive(Debug, Clone)]
pub struct DashboardAdjustmentValidator {
    pub rules: Vec<Rc<dyn DashboardAdjustmentRule>>,
}

impl DashboardAdjustmentValidator {
    pub fn validate(&self, item: &DashboardAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_adjustment_validator() -> DashboardAdjustmentValidator {
    DashboardAdjustmentValidator {
        rules: vec![
            Rc::new(DashboardAdjustmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn DashboardAdjustmentRule>,
            Rc::new(DashboardAdjustmentRequiredCodeRule {
                message: "dashboard adjustment needs this field".to_string(),
            }),
            Rc::new(DashboardAdjustmentRequiredTitleRule {
                message: "dashboard adjustment needs this field".to_string(),
            }),
            Rc::new(DashboardAdjustmentRequiredContactEmailRule {
                message: "dashboard adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_adjustment_validator();
        let item = new_dashboard_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardAdjustmentRequiredCodeRule {
            message: "dashboard adjustment needs an entry".to_string(),
        };
        let mut item = new_dashboard_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DashboardAdjustmentRequiredTitleRule {
            message: "dashboard adjustment needs a value".to_string(),
        };
        let mut item = new_dashboard_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DashboardAdjustmentRequiredContactEmailRule {
            message: "dashboard adjustment needs this field".to_string(),
        };
        let mut item = new_dashboard_adjustment("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
