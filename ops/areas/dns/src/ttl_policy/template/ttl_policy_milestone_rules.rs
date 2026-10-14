use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy milestone must pass. Check returns its problems, or none.
pub trait TtlPolicyMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneRatioRangeRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneCategoryLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneCategoryLengthRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneRequiredTitleRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneAllowedTitleRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneContactEmailLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneContactEmailLengthRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyMilestoneRule for TtlPolicyMilestoneExpiresAtRangeRule {
    fn check(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy milestone.
#[derive(Debug, Clone)]
pub struct TtlPolicyMilestoneValidator {
    pub rules: Vec<Rc<dyn TtlPolicyMilestoneRule>>,
}

impl TtlPolicyMilestoneValidator {
    pub fn validate(&self, item: &TtlPolicyMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_milestone_validator() -> TtlPolicyMilestoneValidator {
    TtlPolicyMilestoneValidator {
        rules: vec![
            Rc::new(TtlPolicyMilestoneRatioRangeRule { min: 9.3, max: 1737.2 }) as Rc<dyn TtlPolicyMilestoneRule>,
            Rc::new(TtlPolicyMilestoneCategoryLengthRule { max_length: 80 }),
            Rc::new(TtlPolicyMilestoneRequiredTitleRule {
                message: "ttl policy milestone needs this field".to_string(),
            }),
            Rc::new(TtlPolicyMilestoneAllowedTitleRule {
                allowed: vec!["capped".to_string(), "lenient".to_string()],
            }),
            Rc::new(TtlPolicyMilestoneContactEmailLengthRule { max_length: 64 }),
            Rc::new(TtlPolicyMilestoneExpiresAtRangeRule { min: 64, max: 255 }),
        ],
    }
}
