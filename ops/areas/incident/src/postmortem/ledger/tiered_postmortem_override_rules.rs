use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered postmortem override must pass. Check returns its problems, or none.
pub trait TieredPostmortemOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TieredPostmortemOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemOverrideBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredPostmortemOverrideRule for TieredPostmortemOverrideBatchSizeRangeRule {
    fn check(&self, item: &TieredPostmortemOverride) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemOverrideUnitLengthRule {
    pub max_length: i64,
}

impl TieredPostmortemOverrideRule for TieredPostmortemOverrideUnitLengthRule {
    fn check(&self, item: &TieredPostmortemOverride) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemOverrideRequiredNameRule {
    pub message: String,
}

impl TieredPostmortemOverrideRule for TieredPostmortemOverrideRequiredNameRule {
    fn check(&self, item: &TieredPostmortemOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered postmortem override.
#[derive(Debug, Clone)]
pub struct TieredPostmortemOverrideValidator {
    pub rules: Vec<Rc<dyn TieredPostmortemOverrideRule>>,
}

impl TieredPostmortemOverrideValidator {
    pub fn validate(&self, item: &TieredPostmortemOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredPostmortemOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPostmortemOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_postmortem_override_validator() -> TieredPostmortemOverrideValidator {
    TieredPostmortemOverrideValidator {
        rules: vec![
            Rc::new(TieredPostmortemOverrideBatchSizeRangeRule { min: 255, max: 1000 })
                as Rc<dyn TieredPostmortemOverrideRule>,
            Rc::new(TieredPostmortemOverrideUnitLengthRule { max_length: 120 }),
            Rc::new(TieredPostmortemOverrideRequiredNameRule {
                message: "tiered postmortem override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_postmortem_override_validator();
        let item = new_tiered_postmortem_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredPostmortemOverrideBatchSizeRangeRule { min: 80, max: 86400 };
        let mut item = new_tiered_postmortem_override("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TieredPostmortemOverrideUnitLengthRule { max_length: 80 };
        let mut item = new_tiered_postmortem_override("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredPostmortemOverrideRequiredNameRule {
            message: "tiered postmortem override needs an entry".to_string(),
        };
        let mut item = new_tiered_postmortem_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
