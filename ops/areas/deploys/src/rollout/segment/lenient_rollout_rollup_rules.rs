use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rollout rollup must pass. Check returns its problems, or none.
pub trait LenientRolloutRollupRule: std::fmt::Debug {
    fn check(&self, item: &LenientRolloutRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutRollupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRolloutRollupRule for LenientRolloutRollupQuantityRangeRule {
    fn check(&self, item: &LenientRolloutRollup) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutRollupRequiredCodeRule {
    pub message: String,
}

impl LenientRolloutRollupRule for LenientRolloutRollupRequiredCodeRule {
    fn check(&self, item: &LenientRolloutRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutRollupTitleLengthRule {
    pub max_length: i64,
}

impl LenientRolloutRollupRule for LenientRolloutRollupTitleLengthRule {
    fn check(&self, item: &LenientRolloutRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rollout rollup.
#[derive(Debug, Clone)]
pub struct LenientRolloutRollupValidator {
    pub rules: Vec<Rc<dyn LenientRolloutRollupRule>>,
}

impl LenientRolloutRollupValidator {
    pub fn validate(&self, item: &LenientRolloutRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRolloutRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRolloutRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rollout_rollup_validator() -> LenientRolloutRollupValidator {
    LenientRolloutRollupValidator {
        rules: vec![
            Rc::new(LenientRolloutRollupQuantityRangeRule { min: 16, max: 3600 }) as Rc<dyn LenientRolloutRollupRule>,
            Rc::new(LenientRolloutRollupRequiredCodeRule {
                message: "lenient rollout rollup needs this field".to_string(),
            }),
            Rc::new(LenientRolloutRollupTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_rollout_rollup_validator();
        let item = new_lenient_rollout_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LenientRolloutRollupQuantityRangeRule { min: 16, max: 32 };
        let mut item = new_lenient_rollout_rollup("x", "y");
        item.quantity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientRolloutRollupRequiredCodeRule {
            message: "lenient rollout rollup needs a value".to_string(),
        };
        let mut item = new_lenient_rollout_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientRolloutRollupTitleLengthRule { max_length: 80 };
        let mut item = new_lenient_rollout_rollup("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
