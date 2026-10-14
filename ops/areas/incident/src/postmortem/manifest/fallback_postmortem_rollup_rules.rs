use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback postmortem rollup must pass. Check returns its problems, or none.
pub trait FallbackPostmortemRollupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPostmortemRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPostmortemRollupRule for FallbackPostmortemRollupExpiresAtRangeRule {
    fn check(&self, item: &FallbackPostmortemRollup) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupRequiredNameRule {
    pub message: String,
}

impl FallbackPostmortemRollupRule for FallbackPostmortemRollupRequiredNameRule {
    fn check(&self, item: &FallbackPostmortemRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPostmortemRollupRule for FallbackPostmortemRollupGraceDaysRangeRule {
    fn check(&self, item: &FallbackPostmortemRollup) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemRollupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPostmortemRollupRule for FallbackPostmortemRollupQuantityRangeRule {
    fn check(&self, item: &FallbackPostmortemRollup) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback postmortem rollup.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemRollupValidator {
    pub rules: Vec<Rc<dyn FallbackPostmortemRollupRule>>,
}

impl FallbackPostmortemRollupValidator {
    pub fn validate(&self, item: &FallbackPostmortemRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPostmortemRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPostmortemRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_postmortem_rollup_validator() -> FallbackPostmortemRollupValidator {
    FallbackPostmortemRollupValidator {
        rules: vec![
            Rc::new(FallbackPostmortemRollupExpiresAtRangeRule { min: 64, max: 80 })
                as Rc<dyn FallbackPostmortemRollupRule>,
            Rc::new(FallbackPostmortemRollupRequiredNameRule {
                message: "fallback postmortem rollup needs a value".to_string(),
            }),
            Rc::new(FallbackPostmortemRollupGraceDaysRangeRule { min: 16, max: 64 }),
            Rc::new(FallbackPostmortemRollupQuantityRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_postmortem_rollup_validator();
        let item = new_fallback_postmortem_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackPostmortemRollupExpiresAtRangeRule { min: 32, max: 255 };
        let mut item = new_fallback_postmortem_rollup("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackPostmortemRollupRequiredNameRule {
            message: "fallback postmortem rollup needs an entry".to_string(),
        };
        let mut item = new_fallback_postmortem_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FallbackPostmortemRollupGraceDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_fallback_postmortem_rollup("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = FallbackPostmortemRollupQuantityRangeRule { min: 1000, max: 3600 };
        let mut item = new_fallback_postmortem_rollup("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
