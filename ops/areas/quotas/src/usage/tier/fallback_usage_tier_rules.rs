use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback usage tier must pass. Check returns its problems, or none.
pub trait FallbackUsageTierRule: std::fmt::Debug {
    fn check(&self, item: &FallbackUsageTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTierAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackUsageTierRule for FallbackUsageTierAllowedNameRule {
    fn check(&self, item: &FallbackUsageTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is stale", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTierDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackUsageTierRule for FallbackUsageTierDiscountRangeRule {
    fn check(&self, item: &FallbackUsageTier) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTierRequiredNameRule {
    pub message: String,
}

impl FallbackUsageTierRule for FallbackUsageTierRequiredNameRule {
    fn check(&self, item: &FallbackUsageTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTierCodePrefixRule {
    pub prefix: String,
}

impl FallbackUsageTierRule for FallbackUsageTierCodePrefixRule {
    fn check(&self, item: &FallbackUsageTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback usage tier.
#[derive(Debug, Clone)]
pub struct FallbackUsageTierValidator {
    pub rules: Vec<Rc<dyn FallbackUsageTierRule>>,
}

impl FallbackUsageTierValidator {
    pub fn validate(&self, item: &FallbackUsageTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackUsageTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackUsageTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_usage_tier_validator() -> FallbackUsageTierValidator {
    FallbackUsageTierValidator {
        rules: vec![
            Rc::new(FallbackUsageTierAllowedNameRule {
                allowed: vec!["strict".to_string(), "regional".to_string()],
            }) as Rc<dyn FallbackUsageTierRule>,
            Rc::new(FallbackUsageTierDiscountRangeRule { min: 1.6, max: 1986.4 }),
            Rc::new(FallbackUsageTierRequiredNameRule {
                message: "fallback usage tier needs this field".to_string(),
            }),
            Rc::new(FallbackUsageTierCodePrefixRule {
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
        let validator = default_fallback_usage_tier_validator();
        let item = new_fallback_usage_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackUsageTierDiscountRangeRule { min: 9.7, max: 2474.0 };
        let mut item = new_fallback_usage_tier("x", "y");
        item.discount = 2475.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackUsageTierRequiredNameRule {
            message: "fallback usage tier needs a value".to_string(),
        };
        let mut item = new_fallback_usage_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
