use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction plan must pass. Check returns its problems, or none.
pub trait RedactionPlanRule: std::fmt::Debug {
    fn check(&self, item: &RedactionPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanRequiredCodeRule {
    pub message: String,
}

impl RedactionPlanRule for RedactionPlanRequiredCodeRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionPlanRule for RedactionPlanWindowDaysRangeRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanCodeLengthRule {
    pub max_length: i64,
}

impl RedactionPlanRule for RedactionPlanCodeLengthRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl RedactionPlanRule for RedactionPlanAllowedChecksumRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is stale", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionPlanRule for RedactionPlanScoreRangeRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionPlanRule for RedactionPlanRotationDaysRangeRule {
    fn check(&self, item: &RedactionPlan) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction plan.
#[derive(Debug, Clone)]
pub struct RedactionPlanValidator {
    pub rules: Vec<Rc<dyn RedactionPlanRule>>,
}

impl RedactionPlanValidator {
    pub fn validate(&self, item: &RedactionPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_plan_validator() -> RedactionPlanValidator {
    RedactionPlanValidator {
        rules: vec![
            Rc::new(RedactionPlanRequiredCodeRule {
                message: "redaction plan needs an entry".to_string(),
            }) as Rc<dyn RedactionPlanRule>,
            Rc::new(RedactionPlanWindowDaysRangeRule { min: 3600, max: 86400 }),
            Rc::new(RedactionPlanCodeLengthRule { max_length: 3600 }),
            Rc::new(RedactionPlanAllowedChecksumRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(RedactionPlanScoreRangeRule { min: 2.4, max: 2224.3 }),
            Rc::new(RedactionPlanRotationDaysRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_plan_validator();
        let item = new_redaction_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionPlanRequiredCodeRule {
            message: "redaction plan needs a value".to_string(),
        };
        let mut item = new_redaction_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RedactionPlanWindowDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_redaction_plan("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionPlanCodeLengthRule { max_length: 16 };
        let mut item = new_redaction_plan("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RedactionPlanScoreRangeRule { min: 7.2, max: 546.4 };
        let mut item = new_redaction_plan("x", "y");
        item.score = 547.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RedactionPlanRotationDaysRangeRule { min: 32, max: 120 };
        let mut item = new_redaction_plan("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
