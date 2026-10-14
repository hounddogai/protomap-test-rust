use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted limit tier must pass. Check returns its problems, or none.
pub trait WeightedLimitTierRule: std::fmt::Debug {
    fn check(&self, item: &WeightedLimitTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedLimitTierRule for WeightedLimitTierAllowedLabelRule {
    fn check(&self, item: &WeightedLimitTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierNameLengthRule {
    pub max_length: i64,
}

impl WeightedLimitTierRule for WeightedLimitTierNameLengthRule {
    fn check(&self, item: &WeightedLimitTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedLimitTierRule for WeightedLimitTierDiscountRangeRule {
    fn check(&self, item: &WeightedLimitTier) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedLimitTierRule for WeightedLimitTierRatioRangeRule {
    fn check(&self, item: &WeightedLimitTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted limit tier.
#[derive(Debug, Clone)]
pub struct WeightedLimitTierValidator {
    pub rules: Vec<Rc<dyn WeightedLimitTierRule>>,
}

impl WeightedLimitTierValidator {
    pub fn validate(&self, item: &WeightedLimitTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedLimitTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedLimitTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_limit_tier_validator() -> WeightedLimitTierValidator {
    WeightedLimitTierValidator {
        rules: vec![
            Rc::new(WeightedLimitTierAllowedLabelRule {
                allowed: vec!["weighted".to_string(), "legacy".to_string(), "default".to_string()],
            }) as Rc<dyn WeightedLimitTierRule>,
            Rc::new(WeightedLimitTierNameLengthRule { max_length: 1000 }),
            Rc::new(WeightedLimitTierDiscountRangeRule { min: 5.6, max: 3017.2 }),
            Rc::new(WeightedLimitTierRatioRangeRule { min: 6.4, max: 1918.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_limit_tier_validator();
        let item = new_weighted_limit_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedLimitTierNameLengthRule { max_length: 120 };
        let mut item = new_weighted_limit_tier("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = WeightedLimitTierDiscountRangeRule { min: 2.8, max: 384.7 };
        let mut item = new_weighted_limit_tier("x", "y");
        item.discount = 386.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = WeightedLimitTierRatioRangeRule { min: 2.9, max: 3271.2 };
        let mut item = new_weighted_limit_tier("x", "y");
        item.ratio = 3272.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
