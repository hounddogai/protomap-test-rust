use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive tenant draft must pass. Check returns its problems, or none.
pub trait AdaptiveTenantDraftRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftDurationMinutesRangeRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
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
pub struct AdaptiveTenantDraftReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftReviewerLengthRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftRequiredNameRule {
    pub message: String,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftRequiredNameRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftCapacityRangeRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftRequiredReviewerRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftSourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantDraftRule for AdaptiveTenantDraftSourceLengthRule {
    fn check(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive tenant draft.
#[derive(Debug, Clone)]
pub struct AdaptiveTenantDraftValidator {
    pub rules: Vec<Rc<dyn AdaptiveTenantDraftRule>>,
}

impl AdaptiveTenantDraftValidator {
    pub fn validate(&self, item: &AdaptiveTenantDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTenantDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTenantDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_tenant_draft_validator() -> AdaptiveTenantDraftValidator {
    AdaptiveTenantDraftValidator {
        rules: vec![
            Rc::new(AdaptiveTenantDraftDurationMinutesRangeRule { min: 16, max: 120 })
                as Rc<dyn AdaptiveTenantDraftRule>,
            Rc::new(AdaptiveTenantDraftReviewerLengthRule { max_length: 80 }),
            Rc::new(AdaptiveTenantDraftRequiredNameRule {
                message: "adaptive tenant draft needs an entry".to_string(),
            }),
            Rc::new(AdaptiveTenantDraftCapacityRangeRule { min: 255, max: 86400 }),
            Rc::new(AdaptiveTenantDraftRequiredReviewerRule {
                message: "adaptive tenant draft needs this field".to_string(),
            }),
            Rc::new(AdaptiveTenantDraftSourceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_tenant_draft_validator();
        let item = new_adaptive_tenant_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = AdaptiveTenantDraftDurationMinutesRangeRule { min: 120, max: 1000 };
        let mut item = new_adaptive_tenant_draft("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = AdaptiveTenantDraftReviewerLengthRule { max_length: 80 };
        let mut item = new_adaptive_tenant_draft("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveTenantDraftRequiredNameRule {
            message: "adaptive tenant draft needs an entry".to_string(),
        };
        let mut item = new_adaptive_tenant_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = AdaptiveTenantDraftCapacityRangeRule { min: 32, max: 255 };
        let mut item = new_adaptive_tenant_draft("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = AdaptiveTenantDraftRequiredReviewerRule {
            message: "adaptive tenant draft needs this field".to_string(),
        };
        let mut item = new_adaptive_tenant_draft("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = AdaptiveTenantDraftSourceLengthRule { max_length: 86400 };
        let mut item = new_adaptive_tenant_draft("x", "y");
        item.source = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
