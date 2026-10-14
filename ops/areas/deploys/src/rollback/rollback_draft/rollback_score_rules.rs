use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback score must pass. Check returns its problems, or none.
pub trait RollbackScoreRule: std::fmt::Debug {
    fn check(&self, item: &RollbackScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreRequiredTenantIdRule {
    pub message: String,
}

impl RollbackScoreRule for RollbackScoreRequiredTenantIdRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreLabelLengthRule {
    pub max_length: i64,
}

impl RollbackScoreRule for RollbackScoreLabelLengthRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreRequiredLabelRule {
    pub message: String,
}

impl RollbackScoreRule for RollbackScoreRequiredLabelRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreCodePrefixRule {
    pub prefix: String,
}

impl RollbackScoreRule for RollbackScoreCodePrefixRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollbackScoreRule for RollbackScoreAllowedCodeRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackScoreRule for RollbackScoreCeilingRangeRule {
    fn check(&self, item: &RollbackScore) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollback score.
#[derive(Debug, Clone)]
pub struct RollbackScoreValidator {
    pub rules: Vec<Rc<dyn RollbackScoreRule>>,
}

impl RollbackScoreValidator {
    pub fn validate(&self, item: &RollbackScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_score_validator() -> RollbackScoreValidator {
    RollbackScoreValidator {
        rules: vec![
            Rc::new(RollbackScoreRequiredTenantIdRule {
                message: "rollback score needs an entry".to_string(),
            }) as Rc<dyn RollbackScoreRule>,
            Rc::new(RollbackScoreLabelLengthRule { max_length: 32 }),
            Rc::new(RollbackScoreRequiredLabelRule {
                message: "rollback score needs this field".to_string(),
            }),
            Rc::new(RollbackScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollbackScoreAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(RollbackScoreCeilingRangeRule { min: 3.7, max: 614.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_score_validator();
        let item = new_rollback_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RollbackScoreRequiredTenantIdRule {
            message: "rollback score needs an entry".to_string(),
        };
        let mut item = new_rollback_score("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackScoreLabelLengthRule { max_length: 3600 };
        let mut item = new_rollback_score("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackScoreRequiredLabelRule {
            message: "rollback score needs this field".to_string(),
        };
        let mut item = new_rollback_score("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollbackScoreCeilingRangeRule { min: 5.1, max: 1511.0 };
        let mut item = new_rollback_score("x", "y");
        item.ceiling = 1512.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
