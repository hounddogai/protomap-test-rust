use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending flow log profile must pass. Check returns its problems, or none.
pub trait PendingFlowLogProfileRule: std::fmt::Debug {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileRequiredCategoryRule {
    pub message: String,
}

impl PendingFlowLogProfileRule for PendingFlowLogProfileRequiredCategoryRule {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileRequiredTitleRule {
    pub message: String,
}

impl PendingFlowLogProfileRule for PendingFlowLogProfileRequiredTitleRule {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileCategoryLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogProfileRule for PendingFlowLogProfileCategoryLengthRule {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileCodePrefixRule {
    pub prefix: String,
}

impl PendingFlowLogProfileRule for PendingFlowLogProfileCodePrefixRule {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogProfileAudienceLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogProfileRule for PendingFlowLogProfileAudienceLengthRule {
    fn check(&self, item: &PendingFlowLogProfile) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending flow log profile.
#[derive(Debug, Clone)]
pub struct PendingFlowLogProfileValidator {
    pub rules: Vec<Rc<dyn PendingFlowLogProfileRule>>,
}

impl PendingFlowLogProfileValidator {
    pub fn validate(&self, item: &PendingFlowLogProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFlowLogProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFlowLogProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_flow_log_profile_validator() -> PendingFlowLogProfileValidator {
    PendingFlowLogProfileValidator {
        rules: vec![
            Rc::new(PendingFlowLogProfileRequiredCategoryRule {
                message: "pending flow log profile needs this field".to_string(),
            }) as Rc<dyn PendingFlowLogProfileRule>,
            Rc::new(PendingFlowLogProfileRequiredTitleRule {
                message: "pending flow log profile needs an entry".to_string(),
            }),
            Rc::new(PendingFlowLogProfileCategoryLengthRule { max_length: 32 }),
            Rc::new(PendingFlowLogProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingFlowLogProfileAudienceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_flow_log_profile_validator();
        let item = new_pending_flow_log_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PendingFlowLogProfileRequiredCategoryRule {
            message: "pending flow log profile needs an entry".to_string(),
        };
        let mut item = new_pending_flow_log_profile("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingFlowLogProfileRequiredTitleRule {
            message: "pending flow log profile needs a value".to_string(),
        };
        let mut item = new_pending_flow_log_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PendingFlowLogProfileCategoryLengthRule { max_length: 32 };
        let mut item = new_pending_flow_log_profile("x", "y");
        item.category = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PendingFlowLogProfileAudienceLengthRule { max_length: 1000 };
        let mut item = new_pending_flow_log_profile("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
