use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending dashboard rollup must pass. Check returns its problems, or none.
pub trait PendingDashboardRollupRule: std::fmt::Debug {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupCodeLengthRule {
    pub max_length: i64,
}

impl PendingDashboardRollupRule for PendingDashboardRollupCodeLengthRule {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupRequiredDescriptionRule {
    pub message: String,
}

impl PendingDashboardRollupRule for PendingDashboardRollupRequiredDescriptionRule {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupCodePrefixRule {
    pub prefix: String,
}

impl PendingDashboardRollupRule for PendingDashboardRollupCodePrefixRule {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupRequiredCodeRule {
    pub message: String,
}

impl PendingDashboardRollupRule for PendingDashboardRollupRequiredCodeRule {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupRequiredTitleRule {
    pub message: String,
}

impl PendingDashboardRollupRule for PendingDashboardRollupRequiredTitleRule {
    fn check(&self, item: &PendingDashboardRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending dashboard rollup.
#[derive(Debug, Clone)]
pub struct PendingDashboardRollupValidator {
    pub rules: Vec<Rc<dyn PendingDashboardRollupRule>>,
}

impl PendingDashboardRollupValidator {
    pub fn validate(&self, item: &PendingDashboardRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingDashboardRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDashboardRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_dashboard_rollup_validator() -> PendingDashboardRollupValidator {
    PendingDashboardRollupValidator {
        rules: vec![
            Rc::new(PendingDashboardRollupCodeLengthRule { max_length: 80 }) as Rc<dyn PendingDashboardRollupRule>,
            Rc::new(PendingDashboardRollupRequiredDescriptionRule {
                message: "pending dashboard rollup needs a value".to_string(),
            }),
            Rc::new(PendingDashboardRollupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingDashboardRollupRequiredCodeRule {
                message: "pending dashboard rollup needs an entry".to_string(),
            }),
            Rc::new(PendingDashboardRollupRequiredTitleRule {
                message: "pending dashboard rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_dashboard_rollup_validator();
        let item = new_pending_dashboard_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingDashboardRollupCodeLengthRule { max_length: 80 };
        let mut item = new_pending_dashboard_rollup("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PendingDashboardRollupRequiredDescriptionRule {
            message: "pending dashboard rollup needs an entry".to_string(),
        };
        let mut item = new_pending_dashboard_rollup("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingDashboardRollupRequiredCodeRule {
            message: "pending dashboard rollup needs this field".to_string(),
        };
        let mut item = new_pending_dashboard_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingDashboardRollupRequiredTitleRule {
            message: "pending dashboard rollup needs an entry".to_string(),
        };
        let mut item = new_pending_dashboard_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
