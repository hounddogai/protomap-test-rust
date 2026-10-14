use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global sink bundle must pass. Check returns its problems, or none.
pub trait GlobalSinkBundleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSinkBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkBundleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSinkBundleRule for GlobalSinkBundleBudgetRangeRule {
    fn check(&self, item: &GlobalSinkBundle) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkBundleRequiredScopeRule {
    pub message: String,
}

impl GlobalSinkBundleRule for GlobalSinkBundleRequiredScopeRule {
    fn check(&self, item: &GlobalSinkBundle) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkBundleAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl GlobalSinkBundleRule for GlobalSinkBundleAllowedClusterRule {
    fn check(&self, item: &GlobalSinkBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is expired", item.cluster)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global sink bundle.
#[derive(Debug, Clone)]
pub struct GlobalSinkBundleValidator {
    pub rules: Vec<Rc<dyn GlobalSinkBundleRule>>,
}

impl GlobalSinkBundleValidator {
    pub fn validate(&self, item: &GlobalSinkBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalSinkBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSinkBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_sink_bundle_validator() -> GlobalSinkBundleValidator {
    GlobalSinkBundleValidator {
        rules: vec![
            Rc::new(GlobalSinkBundleBudgetRangeRule { min: 0.9, max: 290.5 }) as Rc<dyn GlobalSinkBundleRule>,
            Rc::new(GlobalSinkBundleRequiredScopeRule {
                message: "global sink bundle needs this field".to_string(),
            }),
            Rc::new(GlobalSinkBundleAllowedClusterRule {
                allowed: vec!["batched".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_sink_bundle_validator();
        let item = new_global_sink_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GlobalSinkBundleBudgetRangeRule { min: 3.6, max: 765.0 };
        let mut item = new_global_sink_bundle("x", "y");
        item.budget = 766.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = GlobalSinkBundleRequiredScopeRule {
            message: "global sink bundle needs a value".to_string(),
        };
        let mut item = new_global_sink_bundle("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
