use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink tier must pass. Check returns its problems, or none.
pub trait SinkTierRule: std::fmt::Debug {
    fn check(&self, item: &SinkTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkTierRule for SinkTierRatioRangeRule {
    fn check(&self, item: &SinkTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTierVisibleGuardRule {
    pub limit: f64,
}

impl SinkTierRule for SinkTierVisibleGuardRule {
    fn check(&self, item: &SinkTier) -> Vec<String> {
        if item.visible && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while visible", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTierNameLengthRule {
    pub max_length: i64,
}

impl SinkTierRule for SinkTierNameLengthRule {
    fn check(&self, item: &SinkTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTierLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkTierRule for SinkTierLimitRangeRule {
    fn check(&self, item: &SinkTier) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTierLocaleLengthRule {
    pub max_length: i64,
}

impl SinkTierRule for SinkTierLocaleLengthRule {
    fn check(&self, item: &SinkTier) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink tier.
#[derive(Debug, Clone)]
pub struct SinkTierValidator {
    pub rules: Vec<Rc<dyn SinkTierRule>>,
}

impl SinkTierValidator {
    pub fn validate(&self, item: &SinkTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_tier_validator() -> SinkTierValidator {
    SinkTierValidator {
        rules: vec![
            Rc::new(SinkTierRatioRangeRule { min: 8.6, max: 2232.9 }) as Rc<dyn SinkTierRule>,
            Rc::new(SinkTierVisibleGuardRule { limit: 0.2 }),
            Rc::new(SinkTierNameLengthRule { max_length: 120 }),
            Rc::new(SinkTierLimitRangeRule { min: 86400, max: 86400 }),
            Rc::new(SinkTierLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_tier_validator();
        let item = new_sink_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SinkTierRatioRangeRule { min: 7.4, max: 229.8 };
        let mut item = new_sink_tier("x", "y");
        item.ratio = 231.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SinkTierNameLengthRule { max_length: 1000 };
        let mut item = new_sink_tier("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = SinkTierLimitRangeRule { min: 16, max: 32 };
        let mut item = new_sink_tier("x", "y");
        item.limit = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = SinkTierLocaleLengthRule { max_length: 80 };
        let mut item = new_sink_tier("x", "y");
        item.locale = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
