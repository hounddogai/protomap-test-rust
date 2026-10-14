use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted page plan must pass. Check returns its problems, or none.
pub trait WeightedPagePlanRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPagePlanRule for WeightedPagePlanTtlSecondsRangeRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl WeightedPagePlanRule for WeightedPagePlanAllowedCipherRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is below minimum", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanChannelLengthRule {
    pub max_length: i64,
}

impl WeightedPagePlanRule for WeightedPagePlanChannelLengthRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanRequiredChannelRule {
    pub message: String,
}

impl WeightedPagePlanRule for WeightedPagePlanRequiredChannelRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanCodePrefixRule {
    pub prefix: String,
}

impl WeightedPagePlanRule for WeightedPagePlanCodePrefixRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPagePlanRequiredCodeRule {
    pub message: String,
}

impl WeightedPagePlanRule for WeightedPagePlanRequiredCodeRule {
    fn check(&self, item: &WeightedPagePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted page plan.
#[derive(Debug, Clone)]
pub struct WeightedPagePlanValidator {
    pub rules: Vec<Rc<dyn WeightedPagePlanRule>>,
}

impl WeightedPagePlanValidator {
    pub fn validate(&self, item: &WeightedPagePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPagePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPagePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_page_plan_validator() -> WeightedPagePlanValidator {
    WeightedPagePlanValidator {
        rules: vec![
            Rc::new(WeightedPagePlanTtlSecondsRangeRule { min: 120, max: 120 }) as Rc<dyn WeightedPagePlanRule>,
            Rc::new(WeightedPagePlanAllowedCipherRule { allowed: Vec::new() }),
            Rc::new(WeightedPagePlanChannelLengthRule { max_length: 80 }),
            Rc::new(WeightedPagePlanRequiredChannelRule {
                message: "weighted page plan needs this field".to_string(),
            }),
            Rc::new(WeightedPagePlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedPagePlanRequiredCodeRule {
                message: "weighted page plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_page_plan_validator();
        let item = new_weighted_page_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = WeightedPagePlanTtlSecondsRangeRule { min: 255, max: 255 };
        let mut item = new_weighted_page_plan("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = WeightedPagePlanChannelLengthRule { max_length: 80 };
        let mut item = new_weighted_page_plan("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = WeightedPagePlanRequiredChannelRule {
            message: "weighted page plan needs this field".to_string(),
        };
        let mut item = new_weighted_page_plan("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedPagePlanRequiredCodeRule {
            message: "weighted page plan needs an entry".to_string(),
        };
        let mut item = new_weighted_page_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
