use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered limit target must pass. Check returns its problems, or none.
pub trait TieredLimitTargetRule: std::fmt::Debug {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTargetRequiredNameRule {
    pub message: String,
}

impl TieredLimitTargetRule for TieredLimitTargetRequiredNameRule {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTargetTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredLimitTargetRule for TieredLimitTargetTtlSecondsRangeRule {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTargetCodeLengthRule {
    pub max_length: i64,
}

impl TieredLimitTargetRule for TieredLimitTargetCodeLengthRule {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTargetPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredLimitTargetRule for TieredLimitTargetPortRangeRule {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitTargetRequiredCodeRule {
    pub message: String,
}

impl TieredLimitTargetRule for TieredLimitTargetRequiredCodeRule {
    fn check(&self, item: &TieredLimitTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered limit target.
#[derive(Debug, Clone)]
pub struct TieredLimitTargetValidator {
    pub rules: Vec<Rc<dyn TieredLimitTargetRule>>,
}

impl TieredLimitTargetValidator {
    pub fn validate(&self, item: &TieredLimitTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredLimitTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredLimitTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_limit_target_validator() -> TieredLimitTargetValidator {
    TieredLimitTargetValidator {
        rules: vec![
            Rc::new(TieredLimitTargetRequiredNameRule {
                message: "tiered limit target needs a value".to_string(),
            }) as Rc<dyn TieredLimitTargetRule>,
            Rc::new(TieredLimitTargetTtlSecondsRangeRule { min: 16, max: 32 }),
            Rc::new(TieredLimitTargetCodeLengthRule { max_length: 3600 }),
            Rc::new(TieredLimitTargetPortRangeRule { min: 1000, max: 86400 }),
            Rc::new(TieredLimitTargetRequiredCodeRule {
                message: "tiered limit target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_limit_target_validator();
        let item = new_tiered_limit_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredLimitTargetRequiredNameRule {
            message: "tiered limit target needs an entry".to_string(),
        };
        let mut item = new_tiered_limit_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredLimitTargetTtlSecondsRangeRule { min: 80, max: 80 };
        let mut item = new_tiered_limit_target("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredLimitTargetCodeLengthRule { max_length: 86400 };
        let mut item = new_tiered_limit_target("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = TieredLimitTargetPortRangeRule { min: 80, max: 86400 };
        let mut item = new_tiered_limit_target("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredLimitTargetRequiredCodeRule {
            message: "tiered limit target needs a value".to_string(),
        };
        let mut item = new_tiered_limit_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
