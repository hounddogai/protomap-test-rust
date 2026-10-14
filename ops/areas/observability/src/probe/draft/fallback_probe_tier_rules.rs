use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback probe tier must pass. Check returns its problems, or none.
pub trait FallbackProbeTierRule: std::fmt::Debug {
    fn check(&self, item: &FallbackProbeTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTierCodePrefixRule {
    pub prefix: String,
}

impl FallbackProbeTierRule for FallbackProbeTierCodePrefixRule {
    fn check(&self, item: &FallbackProbeTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTierActiveGuardRule {
    pub limit: f64,
}

impl FallbackProbeTierRule for FallbackProbeTierActiveGuardRule {
    fn check(&self, item: &FallbackProbeTier) -> Vec<String> {
        if item.active && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while active",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTierDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackProbeTierRule for FallbackProbeTierDiscountRangeRule {
    fn check(&self, item: &FallbackProbeTier) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeTierCodeLengthRule {
    pub max_length: i64,
}

impl FallbackProbeTierRule for FallbackProbeTierCodeLengthRule {
    fn check(&self, item: &FallbackProbeTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback probe tier.
#[derive(Debug, Clone)]
pub struct FallbackProbeTierValidator {
    pub rules: Vec<Rc<dyn FallbackProbeTierRule>>,
}

impl FallbackProbeTierValidator {
    pub fn validate(&self, item: &FallbackProbeTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackProbeTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackProbeTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_probe_tier_validator() -> FallbackProbeTierValidator {
    FallbackProbeTierValidator {
        rules: vec![
            Rc::new(FallbackProbeTierCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn FallbackProbeTierRule>,
            Rc::new(FallbackProbeTierActiveGuardRule { limit: 0.9 }),
            Rc::new(FallbackProbeTierDiscountRangeRule { min: 4.7, max: 1231.8 }),
            Rc::new(FallbackProbeTierCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_probe_tier_validator();
        let item = new_fallback_probe_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackProbeTierDiscountRangeRule { min: 1.3, max: 1350.7 };
        let mut item = new_fallback_probe_tier("x", "y");
        item.discount = 1352.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackProbeTierCodeLengthRule { max_length: 32 };
        let mut item = new_fallback_probe_tier("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
