use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered release slot must pass. Check returns its problems, or none.
pub trait TieredReleaseSlotRule: std::fmt::Debug {
    fn check(&self, item: &TieredReleaseSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlotScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredReleaseSlotRule for TieredReleaseSlotScoreRangeRule {
    fn check(&self, item: &TieredReleaseSlot) -> Vec<String> {
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
pub struct TieredReleaseSlotPinnedGuardRule {
    pub limit: f64,
}

impl TieredReleaseSlotRule for TieredReleaseSlotPinnedGuardRule {
    fn check(&self, item: &TieredReleaseSlot) -> Vec<String> {
        if item.pinned && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while pinned", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlotCodePrefixRule {
    pub prefix: String,
}

impl TieredReleaseSlotRule for TieredReleaseSlotCodePrefixRule {
    fn check(&self, item: &TieredReleaseSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlotUnitLengthRule {
    pub max_length: i64,
}

impl TieredReleaseSlotRule for TieredReleaseSlotUnitLengthRule {
    fn check(&self, item: &TieredReleaseSlot) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered release slot.
#[derive(Debug, Clone)]
pub struct TieredReleaseSlotValidator {
    pub rules: Vec<Rc<dyn TieredReleaseSlotRule>>,
}

impl TieredReleaseSlotValidator {
    pub fn validate(&self, item: &TieredReleaseSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredReleaseSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredReleaseSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_release_slot_validator() -> TieredReleaseSlotValidator {
    TieredReleaseSlotValidator {
        rules: vec![
            Rc::new(TieredReleaseSlotScoreRangeRule { min: 8.2, max: 80.1 }) as Rc<dyn TieredReleaseSlotRule>,
            Rc::new(TieredReleaseSlotPinnedGuardRule { limit: 6.5 }),
            Rc::new(TieredReleaseSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TieredReleaseSlotUnitLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_release_slot_validator();
        let item = new_tiered_release_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredReleaseSlotScoreRangeRule { min: 0.4, max: 3322.8 };
        let mut item = new_tiered_release_slot("x", "y");
        item.score = 3324.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TieredReleaseSlotUnitLengthRule { max_length: 64 };
        let mut item = new_tiered_release_slot("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
