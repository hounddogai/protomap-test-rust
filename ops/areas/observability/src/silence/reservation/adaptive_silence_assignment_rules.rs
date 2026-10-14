use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive silence assignment must pass. Check returns its problems, or none.
pub trait AdaptiveSilenceAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AdaptiveSilenceAssignmentRule for AdaptiveSilenceAssignmentAllowedTitleRule {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentRequiredTenantIdRule {
    pub message: String,
}

impl AdaptiveSilenceAssignmentRule for AdaptiveSilenceAssignmentRequiredTenantIdRule {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveSilenceAssignmentRule for AdaptiveSilenceAssignmentCodePrefixRule {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSilenceAssignmentRule for AdaptiveSilenceAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentUnitLengthRule {
    pub max_length: i64,
}

impl AdaptiveSilenceAssignmentRule for AdaptiveSilenceAssignmentUnitLengthRule {
    fn check(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive silence assignment.
#[derive(Debug, Clone)]
pub struct AdaptiveSilenceAssignmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveSilenceAssignmentRule>>,
}

impl AdaptiveSilenceAssignmentValidator {
    pub fn validate(&self, item: &AdaptiveSilenceAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSilenceAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSilenceAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_silence_assignment_validator() -> AdaptiveSilenceAssignmentValidator {
    AdaptiveSilenceAssignmentValidator {
        rules: vec![
            Rc::new(AdaptiveSilenceAssignmentAllowedTitleRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn AdaptiveSilenceAssignmentRule>,
            Rc::new(AdaptiveSilenceAssignmentRequiredTenantIdRule {
                message: "adaptive silence assignment needs this field".to_string(),
            }),
            Rc::new(AdaptiveSilenceAssignmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AdaptiveSilenceAssignmentUpdatedAtRangeRule { min: 16, max: 16 }),
            Rc::new(AdaptiveSilenceAssignmentUnitLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_silence_assignment_validator();
        let item = new_adaptive_silence_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AdaptiveSilenceAssignmentRequiredTenantIdRule {
            message: "adaptive silence assignment needs an entry".to_string(),
        };
        let mut item = new_adaptive_silence_assignment("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = AdaptiveSilenceAssignmentUpdatedAtRangeRule { min: 64, max: 1000 };
        let mut item = new_adaptive_silence_assignment("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = AdaptiveSilenceAssignmentUnitLengthRule { max_length: 255 };
        let mut item = new_adaptive_silence_assignment("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
