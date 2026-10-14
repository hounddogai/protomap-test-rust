use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback binding policy must pass. Check returns its problems, or none.
pub trait FallbackBindingPolicyRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicyRequiredAudienceRule {
    pub message: String,
}

impl FallbackBindingPolicyRule for FallbackBindingPolicyRequiredAudienceRule {
    fn check(&self, item: &FallbackBindingPolicy) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicyScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackBindingPolicyRule for FallbackBindingPolicyScoreRangeRule {
    fn check(&self, item: &FallbackBindingPolicy) -> Vec<String> {
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
pub struct FallbackBindingPolicyAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackBindingPolicyRule for FallbackBindingPolicyAllowedNameRule {
    fn check(&self, item: &FallbackBindingPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicyUnitLengthRule {
    pub max_length: i64,
}

impl FallbackBindingPolicyRule for FallbackBindingPolicyUnitLengthRule {
    fn check(&self, item: &FallbackBindingPolicy) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding policy.
#[derive(Debug, Clone)]
pub struct FallbackBindingPolicyValidator {
    pub rules: Vec<Rc<dyn FallbackBindingPolicyRule>>,
}

impl FallbackBindingPolicyValidator {
    pub fn validate(&self, item: &FallbackBindingPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackBindingPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_policy_validator() -> FallbackBindingPolicyValidator {
    FallbackBindingPolicyValidator {
        rules: vec![
            Rc::new(FallbackBindingPolicyRequiredAudienceRule {
                message: "fallback binding policy needs a value".to_string(),
            }) as Rc<dyn FallbackBindingPolicyRule>,
            Rc::new(FallbackBindingPolicyScoreRangeRule { min: 5.4, max: 4426.9 }),
            Rc::new(FallbackBindingPolicyAllowedNameRule {
                allowed: vec!["manual".to_string(), "strict".to_string(), "seasonal".to_string()],
            }),
            Rc::new(FallbackBindingPolicyUnitLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_policy_validator();
        let item = new_fallback_binding_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FallbackBindingPolicyRequiredAudienceRule {
            message: "fallback binding policy needs this field".to_string(),
        };
        let mut item = new_fallback_binding_policy("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FallbackBindingPolicyScoreRangeRule { min: 1.1, max: 3261.6 };
        let mut item = new_fallback_binding_policy("x", "y");
        item.score = 3263.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = FallbackBindingPolicyUnitLengthRule { max_length: 16 };
        let mut item = new_fallback_binding_policy("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
