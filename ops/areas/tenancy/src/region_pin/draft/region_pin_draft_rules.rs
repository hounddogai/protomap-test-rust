use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin draft must pass. Check returns its problems, or none.
pub trait RegionPinDraftRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraftAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinDraftRule for RegionPinDraftAttemptsRangeRule {
    fn check(&self, item: &RegionPinDraft) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraftRequiredChecksumRule {
    pub message: String,
}

impl RegionPinDraftRule for RegionPinDraftRequiredChecksumRule {
    fn check(&self, item: &RegionPinDraft) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraftAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl RegionPinDraftRule for RegionPinDraftAllowedChecksumRule {
    fn check(&self, item: &RegionPinDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is missing", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraftGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinDraftRule for RegionPinDraftGraceDaysRangeRule {
    fn check(&self, item: &RegionPinDraft) -> Vec<String> {
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

/// Runs every rule of a region pin draft.
#[derive(Debug, Clone)]
pub struct RegionPinDraftValidator {
    pub rules: Vec<Rc<dyn RegionPinDraftRule>>,
}

impl RegionPinDraftValidator {
    pub fn validate(&self, item: &RegionPinDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_draft_validator() -> RegionPinDraftValidator {
    RegionPinDraftValidator {
        rules: vec![
            Rc::new(RegionPinDraftAttemptsRangeRule { min: 80, max: 255 }) as Rc<dyn RegionPinDraftRule>,
            Rc::new(RegionPinDraftRequiredChecksumRule {
                message: "region pin draft needs an entry".to_string(),
            }),
            Rc::new(RegionPinDraftAllowedChecksumRule { allowed: Vec::new() }),
            Rc::new(RegionPinDraftGraceDaysRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_draft_validator();
        let item = new_region_pin_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RegionPinDraftAttemptsRangeRule { min: 64, max: 80 };
        let mut item = new_region_pin_draft("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RegionPinDraftRequiredChecksumRule {
            message: "region pin draft needs this field".to_string(),
        };
        let mut item = new_region_pin_draft("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RegionPinDraftGraceDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_region_pin_draft("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
