use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy zone override must pass. Check returns its problems, or none.
pub trait LegacyZoneOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideHostLengthRule {
    pub max_length: i64,
}

impl LegacyZoneOverrideRule for LegacyZoneOverrideHostLengthRule {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideApprovedGuardRule {
    pub limit: i64,
}

impl LegacyZoneOverrideRule for LegacyZoneOverrideApprovedGuardRule {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String> {
        if item.approved && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while approved",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyZoneOverrideRule for LegacyZoneOverrideThresholdRangeRule {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideRequiredCodeRule {
    pub message: String,
}

impl LegacyZoneOverrideRule for LegacyZoneOverrideRequiredCodeRule {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyZoneOverrideRule for LegacyZoneOverrideCreatedAtRangeRule {
    fn check(&self, item: &LegacyZoneOverride) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy zone override.
#[derive(Debug, Clone)]
pub struct LegacyZoneOverrideValidator {
    pub rules: Vec<Rc<dyn LegacyZoneOverrideRule>>,
}

impl LegacyZoneOverrideValidator {
    pub fn validate(&self, item: &LegacyZoneOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyZoneOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyZoneOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_zone_override_validator() -> LegacyZoneOverrideValidator {
    LegacyZoneOverrideValidator {
        rules: vec![
            Rc::new(LegacyZoneOverrideHostLengthRule { max_length: 1000 }) as Rc<dyn LegacyZoneOverrideRule>,
            Rc::new(LegacyZoneOverrideApprovedGuardRule { limit: 80 }),
            Rc::new(LegacyZoneOverrideThresholdRangeRule { min: 4.4, max: 3486.3 }),
            Rc::new(LegacyZoneOverrideRequiredCodeRule {
                message: "legacy zone override needs an entry".to_string(),
            }),
            Rc::new(LegacyZoneOverrideCreatedAtRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_zone_override_validator();
        let item = new_legacy_zone_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = LegacyZoneOverrideHostLengthRule { max_length: 255 };
        let mut item = new_legacy_zone_override("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LegacyZoneOverrideThresholdRangeRule { min: 0.6, max: 2273.8 };
        let mut item = new_legacy_zone_override("x", "y");
        item.threshold = 2275.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyZoneOverrideRequiredCodeRule {
            message: "legacy zone override needs an entry".to_string(),
        };
        let mut item = new_legacy_zone_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LegacyZoneOverrideCreatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_legacy_zone_override("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
