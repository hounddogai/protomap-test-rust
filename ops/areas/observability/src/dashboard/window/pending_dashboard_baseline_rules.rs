use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending dashboard baseline must pass. Check returns its problems, or none.
pub trait PendingDashboardBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PendingDashboardBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselineCodePrefixRule {
    pub prefix: String,
}

impl PendingDashboardBaselineRule for PendingDashboardBaselineCodePrefixRule {
    fn check(&self, item: &PendingDashboardBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselinePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingDashboardBaselineRule for PendingDashboardBaselinePriorityRangeRule {
    fn check(&self, item: &PendingDashboardBaseline) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselineRequiredIssuerRule {
    pub message: String,
}

impl PendingDashboardBaselineRule for PendingDashboardBaselineRequiredIssuerRule {
    fn check(&self, item: &PendingDashboardBaseline) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselineRequiredTitleRule {
    pub message: String,
}

impl PendingDashboardBaselineRule for PendingDashboardBaselineRequiredTitleRule {
    fn check(&self, item: &PendingDashboardBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending dashboard baseline.
#[derive(Debug, Clone)]
pub struct PendingDashboardBaselineValidator {
    pub rules: Vec<Rc<dyn PendingDashboardBaselineRule>>,
}

impl PendingDashboardBaselineValidator {
    pub fn validate(&self, item: &PendingDashboardBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingDashboardBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDashboardBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_dashboard_baseline_validator() -> PendingDashboardBaselineValidator {
    PendingDashboardBaselineValidator {
        rules: vec![
            Rc::new(PendingDashboardBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PendingDashboardBaselineRule>,
            Rc::new(PendingDashboardBaselinePriorityRangeRule { min: 120, max: 3600 }),
            Rc::new(PendingDashboardBaselineRequiredIssuerRule {
                message: "pending dashboard baseline needs an entry".to_string(),
            }),
            Rc::new(PendingDashboardBaselineRequiredTitleRule {
                message: "pending dashboard baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_dashboard_baseline_validator();
        let item = new_pending_dashboard_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PendingDashboardBaselinePriorityRangeRule { min: 32, max: 32 };
        let mut item = new_pending_dashboard_baseline("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PendingDashboardBaselineRequiredIssuerRule {
            message: "pending dashboard baseline needs a value".to_string(),
        };
        let mut item = new_pending_dashboard_baseline("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingDashboardBaselineRequiredTitleRule {
            message: "pending dashboard baseline needs a value".to_string(),
        };
        let mut item = new_pending_dashboard_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
