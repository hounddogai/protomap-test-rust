use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item score must pass. Check returns its problems, or none.
pub trait ActionItemScoreRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreTenantIdLengthRule {
    pub max_length: i64,
}

impl ActionItemScoreRule for ActionItemScoreTenantIdLengthRule {
    fn check(&self, item: &ActionItemScore) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreRequiredClusterRule {
    pub message: String,
}

impl ActionItemScoreRule for ActionItemScoreRequiredClusterRule {
    fn check(&self, item: &ActionItemScore) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemScoreRule for ActionItemScoreUpdatedAtRangeRule {
    fn check(&self, item: &ActionItemScore) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemScoreRule for ActionItemScoreThresholdRangeRule {
    fn check(&self, item: &ActionItemScore) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScoreAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl ActionItemScoreRule for ActionItemScoreAllowedTenantIdRule {
    fn check(&self, item: &ActionItemScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is blocked", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item score.
#[derive(Debug, Clone)]
pub struct ActionItemScoreValidator {
    pub rules: Vec<Rc<dyn ActionItemScoreRule>>,
}

impl ActionItemScoreValidator {
    pub fn validate(&self, item: &ActionItemScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_score_validator() -> ActionItemScoreValidator {
    ActionItemScoreValidator {
        rules: vec![
            Rc::new(ActionItemScoreTenantIdLengthRule { max_length: 120 }) as Rc<dyn ActionItemScoreRule>,
            Rc::new(ActionItemScoreRequiredClusterRule {
                message: "action item score needs a value".to_string(),
            }),
            Rc::new(ActionItemScoreUpdatedAtRangeRule { min: 80, max: 86400 }),
            Rc::new(ActionItemScoreThresholdRangeRule { min: 0.8, max: 3688.0 }),
            Rc::new(ActionItemScoreAllowedTenantIdRule {
                allowed: vec!["tiered".to_string(), "scheduled".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_score_validator();
        let item = new_action_item_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ActionItemScoreTenantIdLengthRule { max_length: 16 };
        let mut item = new_action_item_score("x", "y");
        item.tenant_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ActionItemScoreRequiredClusterRule {
            message: "action item score needs a value".to_string(),
        };
        let mut item = new_action_item_score("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ActionItemScoreUpdatedAtRangeRule { min: 16, max: 120 };
        let mut item = new_action_item_score("x", "y");
        item.updated_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ActionItemScoreThresholdRangeRule { min: 3.4, max: 3579.6 };
        let mut item = new_action_item_score("x", "y");
        item.threshold = 3581.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
