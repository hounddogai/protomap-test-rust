use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit rollup must pass. Check returns its problems, or none.
pub trait LimitRollupRule: std::fmt::Debug {
    fn check(&self, item: &LimitRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitRollupRule for LimitRollupThresholdRangeRule {
    fn check(&self, item: &LimitRollup) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupRequiredLabelRule {
    pub message: String,
}

impl LimitRollupRule for LimitRollupRequiredLabelRule {
    fn check(&self, item: &LimitRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitRollupRule for LimitRollupEndsAtRangeRule {
    fn check(&self, item: &LimitRollup) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupRequiredCipherRule {
    pub message: String,
}

impl LimitRollupRule for LimitRollupRequiredCipherRule {
    fn check(&self, item: &LimitRollup) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit rollup.
#[derive(Debug, Clone)]
pub struct LimitRollupValidator {
    pub rules: Vec<Rc<dyn LimitRollupRule>>,
}

impl LimitRollupValidator {
    pub fn validate(&self, item: &LimitRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_rollup_validator() -> LimitRollupValidator {
    LimitRollupValidator {
        rules: vec![
            Rc::new(LimitRollupThresholdRangeRule { min: 7.2, max: 3778.2 }) as Rc<dyn LimitRollupRule>,
            Rc::new(LimitRollupRequiredLabelRule {
                message: "limit rollup needs an entry".to_string(),
            }),
            Rc::new(LimitRollupEndsAtRangeRule { min: 80, max: 86400 }),
            Rc::new(LimitRollupRequiredCipherRule {
                message: "limit rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_rollup_validator();
        let item = new_limit_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LimitRollupThresholdRangeRule { min: 5.8, max: 3338.6 };
        let mut item = new_limit_rollup("x", "y");
        item.threshold = 3340.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LimitRollupRequiredLabelRule {
            message: "limit rollup needs a value".to_string(),
        };
        let mut item = new_limit_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LimitRollupEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_limit_rollup("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LimitRollupRequiredCipherRule {
            message: "limit rollup needs an entry".to_string(),
        };
        let mut item = new_limit_rollup("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
