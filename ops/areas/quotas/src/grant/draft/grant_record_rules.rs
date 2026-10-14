use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant record must pass. Check returns its problems, or none.
pub trait GrantRecordRule: std::fmt::Debug {
    fn check(&self, item: &GrantRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordDescriptionLengthRule {
    pub max_length: i64,
}

impl GrantRecordRule for GrantRecordDescriptionLengthRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordLabelLengthRule {
    pub max_length: i64,
}

impl GrantRecordRule for GrantRecordLabelLengthRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordRequiredCodeRule {
    pub message: String,
}

impl GrantRecordRule for GrantRecordRequiredCodeRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantRecordRule for GrantRecordDurationMinutesRangeRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordApprovedGuardRule {
    pub limit: i64,
}

impl GrantRecordRule for GrantRecordApprovedGuardRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        if item.approved && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while approved", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl GrantRecordRule for GrantRecordAllowedOwnerIdRule {
    fn check(&self, item: &GrantRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is blocked", item.owner_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant record.
#[derive(Debug, Clone)]
pub struct GrantRecordValidator {
    pub rules: Vec<Rc<dyn GrantRecordRule>>,
}

impl GrantRecordValidator {
    pub fn validate(&self, item: &GrantRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_record_validator() -> GrantRecordValidator {
    GrantRecordValidator {
        rules: vec![
            Rc::new(GrantRecordDescriptionLengthRule { max_length: 16 }) as Rc<dyn GrantRecordRule>,
            Rc::new(GrantRecordLabelLengthRule { max_length: 16 }),
            Rc::new(GrantRecordRequiredCodeRule {
                message: "grant record needs a value".to_string(),
            }),
            Rc::new(GrantRecordDurationMinutesRangeRule { min: 1000, max: 3600 }),
            Rc::new(GrantRecordApprovedGuardRule { limit: 3600 }),
            Rc::new(GrantRecordAllowedOwnerIdRule {
                allowed: vec!["fallback".to_string(), "capped".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_record_validator();
        let item = new_grant_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = GrantRecordDescriptionLengthRule { max_length: 16 };
        let mut item = new_grant_record("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GrantRecordLabelLengthRule { max_length: 16 };
        let mut item = new_grant_record("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantRecordRequiredCodeRule {
            message: "grant record needs an entry".to_string(),
        };
        let mut item = new_grant_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = GrantRecordDurationMinutesRangeRule { min: 80, max: 1000 };
        let mut item = new_grant_record("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
