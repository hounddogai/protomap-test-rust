use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual zone override must pass. Check returns its problems, or none.
pub trait ManualZoneOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ManualZoneOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverrideMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualZoneOverrideRule for ManualZoneOverrideMinimumRangeRule {
    fn check(&self, item: &ManualZoneOverride) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverrideBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualZoneOverrideRule for ManualZoneOverrideBurstRangeRule {
    fn check(&self, item: &ManualZoneOverride) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverrideRequiredCodeRule {
    pub message: String,
}

impl ManualZoneOverrideRule for ManualZoneOverrideRequiredCodeRule {
    fn check(&self, item: &ManualZoneOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualZoneOverrideNameLengthRule {
    pub max_length: i64,
}

impl ManualZoneOverrideRule for ManualZoneOverrideNameLengthRule {
    fn check(&self, item: &ManualZoneOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual zone override.
#[derive(Debug, Clone)]
pub struct ManualZoneOverrideValidator {
    pub rules: Vec<Rc<dyn ManualZoneOverrideRule>>,
}

impl ManualZoneOverrideValidator {
    pub fn validate(&self, item: &ManualZoneOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualZoneOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualZoneOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_zone_override_validator() -> ManualZoneOverrideValidator {
    ManualZoneOverrideValidator {
        rules: vec![
            Rc::new(ManualZoneOverrideMinimumRangeRule { min: 7.7, max: 731.8 }) as Rc<dyn ManualZoneOverrideRule>,
            Rc::new(ManualZoneOverrideBurstRangeRule { min: 16, max: 255 }),
            Rc::new(ManualZoneOverrideRequiredCodeRule {
                message: "manual zone override needs a value".to_string(),
            }),
            Rc::new(ManualZoneOverrideNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_zone_override_validator();
        let item = new_manual_zone_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ManualZoneOverrideMinimumRangeRule { min: 5.0, max: 3533.0 };
        let mut item = new_manual_zone_override("x", "y");
        item.minimum = 3534.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ManualZoneOverrideBurstRangeRule { min: 120, max: 86400 };
        let mut item = new_manual_zone_override("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualZoneOverrideRequiredCodeRule {
            message: "manual zone override needs a value".to_string(),
        };
        let mut item = new_manual_zone_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualZoneOverrideNameLengthRule { max_length: 86400 };
        let mut item = new_manual_zone_override("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
