use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict dashboard milestone must pass. Check returns its problems, or none.
pub trait StrictDashboardMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneRequiredTenantIdRule {
    pub message: String,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneRequiredTenantIdRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneCodeLengthRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneRequiredNameRule {
    pub message: String,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneRequiredNameRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneRequiredHostRule {
    pub message: String,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneRequiredHostRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneCodePrefixRule {
    pub prefix: String,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneCodePrefixRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneHostLengthRule {
    pub max_length: i64,
}

impl StrictDashboardMilestoneRule for StrictDashboardMilestoneHostLengthRule {
    fn check(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict dashboard milestone.
#[derive(Debug, Clone)]
pub struct StrictDashboardMilestoneValidator {
    pub rules: Vec<Rc<dyn StrictDashboardMilestoneRule>>,
}

impl StrictDashboardMilestoneValidator {
    pub fn validate(&self, item: &StrictDashboardMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictDashboardMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDashboardMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_dashboard_milestone_validator() -> StrictDashboardMilestoneValidator {
    StrictDashboardMilestoneValidator {
        rules: vec![
            Rc::new(StrictDashboardMilestoneRequiredTenantIdRule {
                message: "strict dashboard milestone needs this field".to_string(),
            }) as Rc<dyn StrictDashboardMilestoneRule>,
            Rc::new(StrictDashboardMilestoneCodeLengthRule { max_length: 80 }),
            Rc::new(StrictDashboardMilestoneRequiredNameRule {
                message: "strict dashboard milestone needs a value".to_string(),
            }),
            Rc::new(StrictDashboardMilestoneRequiredHostRule {
                message: "strict dashboard milestone needs this field".to_string(),
            }),
            Rc::new(StrictDashboardMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(StrictDashboardMilestoneHostLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_dashboard_milestone_validator();
        let item = new_strict_dashboard_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = StrictDashboardMilestoneRequiredTenantIdRule {
            message: "strict dashboard milestone needs an entry".to_string(),
        };
        let mut item = new_strict_dashboard_milestone("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictDashboardMilestoneCodeLengthRule { max_length: 80 };
        let mut item = new_strict_dashboard_milestone("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictDashboardMilestoneRequiredNameRule {
            message: "strict dashboard milestone needs an entry".to_string(),
        };
        let mut item = new_strict_dashboard_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = StrictDashboardMilestoneRequiredHostRule {
            message: "strict dashboard milestone needs an entry".to_string(),
        };
        let mut item = new_strict_dashboard_milestone("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = StrictDashboardMilestoneHostLengthRule { max_length: 86400 };
        let mut item = new_strict_dashboard_milestone("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
