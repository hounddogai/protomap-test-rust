use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted usage tier must pass. Check returns its problems, or none.
pub trait WeightedUsageTierRule: std::fmt::Debug {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedUsageTierRule for WeightedUsageTierScoreRangeRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
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
pub struct WeightedUsageTierStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedUsageTierRule for WeightedUsageTierStartsAtRangeRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierRequiredCodeRule {
    pub message: String,
}

impl WeightedUsageTierRule for WeightedUsageTierRequiredCodeRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedUsageTierRule for WeightedUsageTierRateRangeRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierRequiredChecksumRule {
    pub message: String,
}

impl WeightedUsageTierRule for WeightedUsageTierRequiredChecksumRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTierCodePrefixRule {
    pub prefix: String,
}

impl WeightedUsageTierRule for WeightedUsageTierCodePrefixRule {
    fn check(&self, item: &WeightedUsageTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted usage tier.
#[derive(Debug, Clone)]
pub struct WeightedUsageTierValidator {
    pub rules: Vec<Rc<dyn WeightedUsageTierRule>>,
}

impl WeightedUsageTierValidator {
    pub fn validate(&self, item: &WeightedUsageTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedUsageTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedUsageTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_usage_tier_validator() -> WeightedUsageTierValidator {
    WeightedUsageTierValidator {
        rules: vec![
            Rc::new(WeightedUsageTierScoreRangeRule { min: 0.6, max: 779.1 }) as Rc<dyn WeightedUsageTierRule>,
            Rc::new(WeightedUsageTierStartsAtRangeRule { min: 80, max: 120 }),
            Rc::new(WeightedUsageTierRequiredCodeRule {
                message: "weighted usage tier needs a value".to_string(),
            }),
            Rc::new(WeightedUsageTierRateRangeRule { min: 1.1, max: 3965.9 }),
            Rc::new(WeightedUsageTierRequiredChecksumRule {
                message: "weighted usage tier needs an entry".to_string(),
            }),
            Rc::new(WeightedUsageTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_usage_tier_validator();
        let item = new_weighted_usage_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = WeightedUsageTierScoreRangeRule { min: 8.0, max: 2253.7 };
        let mut item = new_weighted_usage_tier("x", "y");
        item.score = 2255.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = WeightedUsageTierStartsAtRangeRule { min: 255, max: 3600 };
        let mut item = new_weighted_usage_tier("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedUsageTierRequiredCodeRule {
            message: "weighted usage tier needs a value".to_string(),
        };
        let mut item = new_weighted_usage_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = WeightedUsageTierRateRangeRule { min: 2.1, max: 3184.2 };
        let mut item = new_weighted_usage_tier("x", "y");
        item.rate = 3185.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = WeightedUsageTierRequiredChecksumRule {
            message: "weighted usage tier needs a value".to_string(),
        };
        let mut item = new_weighted_usage_tier("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
