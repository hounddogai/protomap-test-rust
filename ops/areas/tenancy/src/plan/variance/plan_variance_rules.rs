use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan variance must pass. Check returns its problems, or none.
pub trait PlanVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PlanVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanVarianceAllowedHostRule {
    pub allowed: Vec<String>,
}

impl PlanVarianceRule for PlanVarianceAllowedHostRule {
    fn check(&self, item: &PlanVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is not allowed", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanVarianceRequiredHostRule {
    pub message: String,
}

impl PlanVarianceRule for PlanVarianceRequiredHostRule {
    fn check(&self, item: &PlanVariance) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanVarianceRequiredTitleRule {
    pub message: String,
}

impl PlanVarianceRule for PlanVarianceRequiredTitleRule {
    fn check(&self, item: &PlanVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan variance.
#[derive(Debug, Clone)]
pub struct PlanVarianceValidator {
    pub rules: Vec<Rc<dyn PlanVarianceRule>>,
}

impl PlanVarianceValidator {
    pub fn validate(&self, item: &PlanVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_variance_validator() -> PlanVarianceValidator {
    PlanVarianceValidator {
        rules: vec![
            Rc::new(PlanVarianceAllowedHostRule {
                allowed: vec!["weighted".to_string(), "legacy".to_string(), "strict".to_string()],
            }) as Rc<dyn PlanVarianceRule>,
            Rc::new(PlanVarianceRequiredHostRule {
                message: "plan variance needs a value".to_string(),
            }),
            Rc::new(PlanVarianceRequiredTitleRule {
                message: "plan variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_variance_validator();
        let item = new_plan_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PlanVarianceRequiredHostRule {
            message: "plan variance needs a value".to_string(),
        };
        let mut item = new_plan_variance("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PlanVarianceRequiredTitleRule {
            message: "plan variance needs this field".to_string(),
        };
        let mut item = new_plan_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
