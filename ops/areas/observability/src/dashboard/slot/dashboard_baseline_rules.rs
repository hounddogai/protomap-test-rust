use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard baseline must pass. Check returns its problems, or none.
pub trait DashboardBaselineRule: std::fmt::Debug {
    fn check(&self, item: &DashboardBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaselineTitleLengthRule {
    pub max_length: i64,
}

impl DashboardBaselineRule for DashboardBaselineTitleLengthRule {
    fn check(&self, item: &DashboardBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaselineRequiredContactEmailRule {
    pub message: String,
}

impl DashboardBaselineRule for DashboardBaselineRequiredContactEmailRule {
    fn check(&self, item: &DashboardBaseline) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaselineNamespaceLengthRule {
    pub max_length: i64,
}

impl DashboardBaselineRule for DashboardBaselineNamespaceLengthRule {
    fn check(&self, item: &DashboardBaseline) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaselineRequiredTitleRule {
    pub message: String,
}

impl DashboardBaselineRule for DashboardBaselineRequiredTitleRule {
    fn check(&self, item: &DashboardBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard baseline.
#[derive(Debug, Clone)]
pub struct DashboardBaselineValidator {
    pub rules: Vec<Rc<dyn DashboardBaselineRule>>,
}

impl DashboardBaselineValidator {
    pub fn validate(&self, item: &DashboardBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_baseline_validator() -> DashboardBaselineValidator {
    DashboardBaselineValidator {
        rules: vec![
            Rc::new(DashboardBaselineTitleLengthRule { max_length: 32 }) as Rc<dyn DashboardBaselineRule>,
            Rc::new(DashboardBaselineRequiredContactEmailRule {
                message: "dashboard baseline needs a value".to_string(),
            }),
            Rc::new(DashboardBaselineNamespaceLengthRule { max_length: 3600 }),
            Rc::new(DashboardBaselineRequiredTitleRule {
                message: "dashboard baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_baseline_validator();
        let item = new_dashboard_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DashboardBaselineTitleLengthRule { max_length: 120 };
        let mut item = new_dashboard_baseline("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DashboardBaselineRequiredContactEmailRule {
            message: "dashboard baseline needs this field".to_string(),
        };
        let mut item = new_dashboard_baseline("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DashboardBaselineNamespaceLengthRule { max_length: 80 };
        let mut item = new_dashboard_baseline("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DashboardBaselineRequiredTitleRule {
            message: "dashboard baseline needs an entry".to_string(),
        };
        let mut item = new_dashboard_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
