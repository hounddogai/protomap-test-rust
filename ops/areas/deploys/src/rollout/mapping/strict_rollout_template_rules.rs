use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict rollout template must pass. Check returns its problems, or none.
pub trait StrictRolloutTemplateRule: std::fmt::Debug {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateAllowedCodeRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateThresholdRangeRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateRatioRangeRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateRotationDaysRangeRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateCodeLengthRule {
    pub max_length: i64,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateCodeLengthRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateCodePrefixRule {
    pub prefix: String,
}

impl StrictRolloutTemplateRule for StrictRolloutTemplateCodePrefixRule {
    fn check(&self, item: &StrictRolloutTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict rollout template.
#[derive(Debug, Clone)]
pub struct StrictRolloutTemplateValidator {
    pub rules: Vec<Rc<dyn StrictRolloutTemplateRule>>,
}

impl StrictRolloutTemplateValidator {
    pub fn validate(&self, item: &StrictRolloutTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRolloutTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRolloutTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rollout_template_validator() -> StrictRolloutTemplateValidator {
    StrictRolloutTemplateValidator {
        rules: vec![
            Rc::new(StrictRolloutTemplateAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn StrictRolloutTemplateRule>,
            Rc::new(StrictRolloutTemplateThresholdRangeRule { min: 0.0, max: 649.1 }),
            Rc::new(StrictRolloutTemplateRatioRangeRule { min: 1.7, max: 3986.1 }),
            Rc::new(StrictRolloutTemplateRotationDaysRangeRule { min: 16, max: 255 }),
            Rc::new(StrictRolloutTemplateCodeLengthRule { max_length: 64 }),
            Rc::new(StrictRolloutTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
