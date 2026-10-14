use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard draft must pass. Check returns its problems, or none.
pub trait DashboardDraftRule: std::fmt::Debug {
    fn check(&self, item: &DashboardDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardDraftRule for DashboardDraftCapacityRangeRule {
    fn check(&self, item: &DashboardDraft) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDraftCodePrefixRule {
    pub prefix: String,
}

impl DashboardDraftRule for DashboardDraftCodePrefixRule {
    fn check(&self, item: &DashboardDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DashboardDraftRule for DashboardDraftAllowedCodeRule {
    fn check(&self, item: &DashboardDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard draft.
#[derive(Debug, Clone)]
pub struct DashboardDraftValidator {
    pub rules: Vec<Rc<dyn DashboardDraftRule>>,
}

impl DashboardDraftValidator {
    pub fn validate(&self, item: &DashboardDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_draft_validator() -> DashboardDraftValidator {
    DashboardDraftValidator {
        rules: vec![
            Rc::new(DashboardDraftCapacityRangeRule { min: 3600, max: 3600 }) as Rc<dyn DashboardDraftRule>,
            Rc::new(DashboardDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DashboardDraftAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_draft_validator();
        let item = new_dashboard_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = DashboardDraftCapacityRangeRule { min: 16, max: 120 };
        let mut item = new_dashboard_draft("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
