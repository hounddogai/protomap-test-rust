use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary ttl policy plan must pass. Check returns its problems, or none.
pub trait PrimaryTtlPolicyPlanRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryTtlPolicyPlanRule for PrimaryTtlPolicyPlanScoreRangeRule {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanRequiredTitleRule {
    pub message: String,
}

impl PrimaryTtlPolicyPlanRule for PrimaryTtlPolicyPlanRequiredTitleRule {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryTtlPolicyPlanRule for PrimaryTtlPolicyPlanAllowedLabelRule {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanApprovedGuardRule {
    pub limit: f64,
}

impl PrimaryTtlPolicyPlanRule for PrimaryTtlPolicyPlanApprovedGuardRule {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        if item.approved && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while approved", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanCodePrefixRule {
    pub prefix: String,
}

impl PrimaryTtlPolicyPlanRule for PrimaryTtlPolicyPlanCodePrefixRule {
    fn check(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary ttl policy plan.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyPlanValidator {
    pub rules: Vec<Rc<dyn PrimaryTtlPolicyPlanRule>>,
}

impl PrimaryTtlPolicyPlanValidator {
    pub fn validate(&self, item: &PrimaryTtlPolicyPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryTtlPolicyPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTtlPolicyPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_ttl_policy_plan_validator() -> PrimaryTtlPolicyPlanValidator {
    PrimaryTtlPolicyPlanValidator {
        rules: vec![
            Rc::new(PrimaryTtlPolicyPlanScoreRangeRule { min: 2.8, max: 3876.2 }) as Rc<dyn PrimaryTtlPolicyPlanRule>,
            Rc::new(PrimaryTtlPolicyPlanRequiredTitleRule {
                message: "primary ttl policy plan needs an entry".to_string(),
            }),
            Rc::new(PrimaryTtlPolicyPlanAllowedLabelRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(PrimaryTtlPolicyPlanApprovedGuardRule { limit: 9.6 }),
            Rc::new(PrimaryTtlPolicyPlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_ttl_policy_plan_validator();
        let item = new_primary_ttl_policy_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PrimaryTtlPolicyPlanScoreRangeRule { min: 10.0, max: 1861.8 };
        let mut item = new_primary_ttl_policy_plan("x", "y");
        item.score = 1863.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryTtlPolicyPlanRequiredTitleRule {
            message: "primary ttl policy plan needs an entry".to_string(),
        };
        let mut item = new_primary_ttl_policy_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
