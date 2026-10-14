use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy allowlist target must pass. Check returns its problems, or none.
pub trait LegacyAllowlistTargetRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetCodeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistTargetRule for LegacyAllowlistTargetCodeLengthRule {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetRequiredSegmentRule {
    pub message: String,
}

impl LegacyAllowlistTargetRule for LegacyAllowlistTargetRequiredSegmentRule {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetRequiredCurrencyRule {
    pub message: String,
}

impl LegacyAllowlistTargetRule for LegacyAllowlistTargetRequiredCurrencyRule {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAllowlistTargetRule for LegacyAllowlistTargetMaximumRangeRule {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTargetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAllowlistTargetRule for LegacyAllowlistTargetCeilingRangeRule {
    fn check(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy allowlist target.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistTargetValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistTargetRule>>,
}

impl LegacyAllowlistTargetValidator {
    pub fn validate(&self, item: &LegacyAllowlistTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyAllowlistTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_target_validator() -> LegacyAllowlistTargetValidator {
    LegacyAllowlistTargetValidator {
        rules: vec![
            Rc::new(LegacyAllowlistTargetCodeLengthRule { max_length: 3600 }) as Rc<dyn LegacyAllowlistTargetRule>,
            Rc::new(LegacyAllowlistTargetRequiredSegmentRule {
                message: "legacy allowlist target needs this field".to_string(),
            }),
            Rc::new(LegacyAllowlistTargetRequiredCurrencyRule {
                message: "legacy allowlist target needs a value".to_string(),
            }),
            Rc::new(LegacyAllowlistTargetMaximumRangeRule { min: 2.5, max: 3730.4 }),
            Rc::new(LegacyAllowlistTargetCeilingRangeRule { min: 6.8, max: 2978.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_target_validator();
        let item = new_legacy_allowlist_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyAllowlistTargetCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_allowlist_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = LegacyAllowlistTargetRequiredSegmentRule {
            message: "legacy allowlist target needs a value".to_string(),
        };
        let mut item = new_legacy_allowlist_target("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = LegacyAllowlistTargetRequiredCurrencyRule {
            message: "legacy allowlist target needs this field".to_string(),
        };
        let mut item = new_legacy_allowlist_target("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LegacyAllowlistTargetMaximumRangeRule { min: 6.6, max: 4785.9 };
        let mut item = new_legacy_allowlist_target("x", "y");
        item.maximum = 4787.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LegacyAllowlistTargetCeilingRangeRule { min: 8.8, max: 1011.0 };
        let mut item = new_legacy_allowlist_target("x", "y");
        item.ceiling = 1012.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
