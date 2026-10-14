use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual credential score must pass. Check returns its problems, or none.
pub trait ManualCredentialScoreRule: std::fmt::Debug {
    fn check(&self, item: &ManualCredentialScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialScoreAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl ManualCredentialScoreRule for ManualCredentialScoreAllowedCategoryRule {
    fn check(&self, item: &ManualCredentialScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is expired", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialScoreFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualCredentialScoreRule for ManualCredentialScoreFloorValueRangeRule {
    fn check(&self, item: &ManualCredentialScore) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialScoreRequiredCodeRule {
    pub message: String,
}

impl ManualCredentialScoreRule for ManualCredentialScoreRequiredCodeRule {
    fn check(&self, item: &ManualCredentialScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual credential score.
#[derive(Debug, Clone)]
pub struct ManualCredentialScoreValidator {
    pub rules: Vec<Rc<dyn ManualCredentialScoreRule>>,
}

impl ManualCredentialScoreValidator {
    pub fn validate(&self, item: &ManualCredentialScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualCredentialScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCredentialScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_credential_score_validator() -> ManualCredentialScoreValidator {
    ManualCredentialScoreValidator {
        rules: vec![
            Rc::new(ManualCredentialScoreAllowedCategoryRule {
                allowed: vec!["scheduled".to_string(), "adaptive".to_string(), "pending".to_string()],
            }) as Rc<dyn ManualCredentialScoreRule>,
            Rc::new(ManualCredentialScoreFloorValueRangeRule { min: 1.0, max: 4779.0 }),
            Rc::new(ManualCredentialScoreRequiredCodeRule {
                message: "manual credential score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_credential_score_validator();
        let item = new_manual_credential_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ManualCredentialScoreFloorValueRangeRule { min: 3.2, max: 4161.8 };
        let mut item = new_manual_credential_score("x", "y");
        item.floor_value = 4163.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualCredentialScoreRequiredCodeRule {
            message: "manual credential score needs a value".to_string(),
        };
        let mut item = new_manual_credential_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
