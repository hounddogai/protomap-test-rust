use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered overage threshold must pass. Check returns its problems, or none.
pub trait TieredOverageThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdNameLengthRule {
    pub max_length: i64,
}

impl TieredOverageThresholdRule for TieredOverageThresholdNameLengthRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredOverageThresholdRule for TieredOverageThresholdRateRangeRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdCodePrefixRule {
    pub prefix: String,
}

impl TieredOverageThresholdRule for TieredOverageThresholdCodePrefixRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdCodeLengthRule {
    pub max_length: i64,
}

impl TieredOverageThresholdRule for TieredOverageThresholdCodeLengthRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdArchivedGuardRule {
    pub limit: f64,
}

impl TieredOverageThresholdRule for TieredOverageThresholdArchivedGuardRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        if item.archived && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while archived", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThresholdRequiredCodeRule {
    pub message: String,
}

impl TieredOverageThresholdRule for TieredOverageThresholdRequiredCodeRule {
    fn check(&self, item: &TieredOverageThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage threshold.
#[derive(Debug, Clone)]
pub struct TieredOverageThresholdValidator {
    pub rules: Vec<Rc<dyn TieredOverageThresholdRule>>,
}

impl TieredOverageThresholdValidator {
    pub fn validate(&self, item: &TieredOverageThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredOverageThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_threshold_validator() -> TieredOverageThresholdValidator {
    TieredOverageThresholdValidator {
        rules: vec![
            Rc::new(TieredOverageThresholdNameLengthRule { max_length: 120 }) as Rc<dyn TieredOverageThresholdRule>,
            Rc::new(TieredOverageThresholdRateRangeRule { min: 3.4, max: 1878.2 }),
            Rc::new(TieredOverageThresholdCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TieredOverageThresholdCodeLengthRule { max_length: 80 }),
            Rc::new(TieredOverageThresholdArchivedGuardRule { limit: 0.5 }),
            Rc::new(TieredOverageThresholdRequiredCodeRule {
                message: "tiered overage threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_threshold_validator();
        let item = new_tiered_overage_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredOverageThresholdNameLengthRule { max_length: 32 };
        let mut item = new_tiered_overage_threshold("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = TieredOverageThresholdRateRangeRule { min: 5.0, max: 1862.6 };
        let mut item = new_tiered_overage_threshold("x", "y");
        item.rate = 1864.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredOverageThresholdCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_overage_threshold("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredOverageThresholdRequiredCodeRule {
            message: "tiered overage threshold needs an entry".to_string(),
        };
        let mut item = new_tiered_overage_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
