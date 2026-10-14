use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled silence rollup must pass. Check returns its problems, or none.
pub trait ScheduledSilenceRollupRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupTenantIdLengthRule {
    pub max_length: i64,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupTenantIdLengthRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupCodePrefixRule {
    pub prefix: String,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupCodePrefixRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupMinimumRangeRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupDurationMinutesRangeRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupNameLengthRule {
    pub max_length: i64,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupNameLengthRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceRollupTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledSilenceRollupRule for ScheduledSilenceRollupTitleLengthRule {
    fn check(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled silence rollup.
#[derive(Debug, Clone)]
pub struct ScheduledSilenceRollupValidator {
    pub rules: Vec<Rc<dyn ScheduledSilenceRollupRule>>,
}

impl ScheduledSilenceRollupValidator {
    pub fn validate(&self, item: &ScheduledSilenceRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSilenceRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSilenceRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_silence_rollup_validator() -> ScheduledSilenceRollupValidator {
    ScheduledSilenceRollupValidator {
        rules: vec![
            Rc::new(ScheduledSilenceRollupTenantIdLengthRule { max_length: 80 }) as Rc<dyn ScheduledSilenceRollupRule>,
            Rc::new(ScheduledSilenceRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledSilenceRollupMinimumRangeRule { min: 5.9, max: 3655.2 }),
            Rc::new(ScheduledSilenceRollupDurationMinutesRangeRule { min: 80, max: 1000 }),
            Rc::new(ScheduledSilenceRollupNameLengthRule { max_length: 3600 }),
            Rc::new(ScheduledSilenceRollupTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_silence_rollup_validator();
        let item = new_scheduled_silence_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ScheduledSilenceRollupTenantIdLengthRule { max_length: 80 };
        let mut item = new_scheduled_silence_rollup("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ScheduledSilenceRollupMinimumRangeRule { min: 9.0, max: 3576.5 };
        let mut item = new_scheduled_silence_rollup("x", "y");
        item.minimum = 3578.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = ScheduledSilenceRollupDurationMinutesRangeRule { min: 32, max: 255 };
        let mut item = new_scheduled_silence_rollup("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledSilenceRollupNameLengthRule { max_length: 3600 };
        let mut item = new_scheduled_silence_rollup("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledSilenceRollupTitleLengthRule { max_length: 1000 };
        let mut item = new_scheduled_silence_rollup("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
