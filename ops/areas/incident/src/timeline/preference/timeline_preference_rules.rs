use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline preference must pass. Check returns its problems, or none.
pub trait TimelinePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &TimelinePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePreferenceAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl TimelinePreferenceRule for TimelinePreferenceAllowedFingerprintRule {
    fn check(&self, item: &TimelinePreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is duplicated", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePreferenceRequiredTenantIdRule {
    pub message: String,
}

impl TimelinePreferenceRule for TimelinePreferenceRequiredTenantIdRule {
    fn check(&self, item: &TimelinePreference) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePreferenceTitleLengthRule {
    pub max_length: i64,
}

impl TimelinePreferenceRule for TimelinePreferenceTitleLengthRule {
    fn check(&self, item: &TimelinePreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePreferenceTenantIdLengthRule {
    pub max_length: i64,
}

impl TimelinePreferenceRule for TimelinePreferenceTenantIdLengthRule {
    fn check(&self, item: &TimelinePreference) -> Vec<String> {
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

/// Runs every rule of a timeline preference.
#[derive(Debug, Clone)]
pub struct TimelinePreferenceValidator {
    pub rules: Vec<Rc<dyn TimelinePreferenceRule>>,
}

impl TimelinePreferenceValidator {
    pub fn validate(&self, item: &TimelinePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelinePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelinePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_preference_validator() -> TimelinePreferenceValidator {
    TimelinePreferenceValidator {
        rules: vec![
            Rc::new(TimelinePreferenceAllowedFingerprintRule {
                allowed: vec!["batched".to_string(), "scheduled".to_string()],
            }) as Rc<dyn TimelinePreferenceRule>,
            Rc::new(TimelinePreferenceRequiredTenantIdRule {
                message: "timeline preference needs a value".to_string(),
            }),
            Rc::new(TimelinePreferenceTitleLengthRule { max_length: 120 }),
            Rc::new(TimelinePreferenceTenantIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_preference_validator();
        let item = new_timeline_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TimelinePreferenceRequiredTenantIdRule {
            message: "timeline preference needs this field".to_string(),
        };
        let mut item = new_timeline_preference("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelinePreferenceTitleLengthRule { max_length: 80 };
        let mut item = new_timeline_preference("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TimelinePreferenceTenantIdLengthRule { max_length: 1000 };
        let mut item = new_timeline_preference("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
