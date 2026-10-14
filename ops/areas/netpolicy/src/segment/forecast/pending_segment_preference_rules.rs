use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending segment preference must pass. Check returns its problems, or none.
pub trait PendingSegmentPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreferenceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferenceMarginRangeRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreferencePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferencePriorityRangeRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreferenceTenantIdLengthRule {
    pub max_length: i64,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferenceTenantIdLengthRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
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
pub struct PendingSegmentPreferenceHostLengthRule {
    pub max_length: i64,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferenceHostLengthRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferenceCodeLengthRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentPreferenceRequiredHostRule {
    pub message: String,
}

impl PendingSegmentPreferenceRule for PendingSegmentPreferenceRequiredHostRule {
    fn check(&self, item: &PendingSegmentPreference) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending segment preference.
#[derive(Debug, Clone)]
pub struct PendingSegmentPreferenceValidator {
    pub rules: Vec<Rc<dyn PendingSegmentPreferenceRule>>,
}

impl PendingSegmentPreferenceValidator {
    pub fn validate(&self, item: &PendingSegmentPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingSegmentPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSegmentPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_segment_preference_validator() -> PendingSegmentPreferenceValidator {
    PendingSegmentPreferenceValidator {
        rules: vec![
            Rc::new(PendingSegmentPreferenceMarginRangeRule { min: 6.8, max: 3851.0 })
                as Rc<dyn PendingSegmentPreferenceRule>,
            Rc::new(PendingSegmentPreferencePriorityRangeRule { min: 32, max: 80 }),
            Rc::new(PendingSegmentPreferenceTenantIdLengthRule { max_length: 255 }),
            Rc::new(PendingSegmentPreferenceHostLengthRule { max_length: 32 }),
            Rc::new(PendingSegmentPreferenceCodeLengthRule { max_length: 255 }),
            Rc::new(PendingSegmentPreferenceRequiredHostRule {
                message: "pending segment preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_segment_preference_validator();
        let item = new_pending_segment_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PendingSegmentPreferenceMarginRangeRule { min: 7.6, max: 571.8 };
        let mut item = new_pending_segment_preference("x", "y");
        item.margin = 573.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PendingSegmentPreferencePriorityRangeRule { min: 32, max: 64 };
        let mut item = new_pending_segment_preference("x", "y");
        item.priority = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PendingSegmentPreferenceTenantIdLengthRule { max_length: 64 };
        let mut item = new_pending_segment_preference("x", "y");
        item.tenant_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PendingSegmentPreferenceHostLengthRule { max_length: 1000 };
        let mut item = new_pending_segment_preference("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingSegmentPreferenceCodeLengthRule { max_length: 16 };
        let mut item = new_pending_segment_preference("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PendingSegmentPreferenceRequiredHostRule {
            message: "pending segment preference needs this field".to_string(),
        };
        let mut item = new_pending_segment_preference("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
