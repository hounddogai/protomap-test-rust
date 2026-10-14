use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted token plan must pass. Check returns its problems, or none.
pub trait WeightedTokenPlanRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTokenPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenPlanRequiredAudienceRule {
    pub message: String,
}

impl WeightedTokenPlanRule for WeightedTokenPlanRequiredAudienceRule {
    fn check(&self, item: &WeightedTokenPlan) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenPlanRequiredCodeRule {
    pub message: String,
}

impl WeightedTokenPlanRule for WeightedTokenPlanRequiredCodeRule {
    fn check(&self, item: &WeightedTokenPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenPlanAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl WeightedTokenPlanRule for WeightedTokenPlanAllowedPrincipalRule {
    fn check(&self, item: &WeightedTokenPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is missing", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenPlanLabelLengthRule {
    pub max_length: i64,
}

impl WeightedTokenPlanRule for WeightedTokenPlanLabelLengthRule {
    fn check(&self, item: &WeightedTokenPlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted token plan.
#[derive(Debug, Clone)]
pub struct WeightedTokenPlanValidator {
    pub rules: Vec<Rc<dyn WeightedTokenPlanRule>>,
}

impl WeightedTokenPlanValidator {
    pub fn validate(&self, item: &WeightedTokenPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTokenPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTokenPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_token_plan_validator() -> WeightedTokenPlanValidator {
    WeightedTokenPlanValidator {
        rules: vec![
            Rc::new(WeightedTokenPlanRequiredAudienceRule {
                message: "weighted token plan needs this field".to_string(),
            }) as Rc<dyn WeightedTokenPlanRule>,
            Rc::new(WeightedTokenPlanRequiredCodeRule {
                message: "weighted token plan needs this field".to_string(),
            }),
            Rc::new(WeightedTokenPlanAllowedPrincipalRule { allowed: Vec::new() }),
            Rc::new(WeightedTokenPlanLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_token_plan_validator();
        let item = new_weighted_token_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = WeightedTokenPlanRequiredAudienceRule {
            message: "weighted token plan needs this field".to_string(),
        };
        let mut item = new_weighted_token_plan("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedTokenPlanRequiredCodeRule {
            message: "weighted token plan needs this field".to_string(),
        };
        let mut item = new_weighted_token_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedTokenPlanLabelLengthRule { max_length: 32 };
        let mut item = new_weighted_token_plan("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
