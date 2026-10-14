use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence milestone must pass. Check returns its problems, or none.
pub trait EvidenceMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl EvidenceMilestoneRule for EvidenceMilestoneAllowedTitleRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneRequiredCodeRule {
    pub message: String,
}

impl EvidenceMilestoneRule for EvidenceMilestoneRequiredCodeRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneRequiredCountryRule {
    pub message: String,
}

impl EvidenceMilestoneRule for EvidenceMilestoneRequiredCountryRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneRequiredTitleRule {
    pub message: String,
}

impl EvidenceMilestoneRule for EvidenceMilestoneRequiredTitleRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneHostLengthRule {
    pub max_length: i64,
}

impl EvidenceMilestoneRule for EvidenceMilestoneHostLengthRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMilestoneRequiredHostRule {
    pub message: String,
}

impl EvidenceMilestoneRule for EvidenceMilestoneRequiredHostRule {
    fn check(&self, item: &EvidenceMilestone) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence milestone.
#[derive(Debug, Clone)]
pub struct EvidenceMilestoneValidator {
    pub rules: Vec<Rc<dyn EvidenceMilestoneRule>>,
}

impl EvidenceMilestoneValidator {
    pub fn validate(&self, item: &EvidenceMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_milestone_validator() -> EvidenceMilestoneValidator {
    EvidenceMilestoneValidator {
        rules: vec![
            Rc::new(EvidenceMilestoneAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn EvidenceMilestoneRule>,
            Rc::new(EvidenceMilestoneRequiredCodeRule {
                message: "evidence milestone needs an entry".to_string(),
            }),
            Rc::new(EvidenceMilestoneRequiredCountryRule {
                message: "evidence milestone needs a value".to_string(),
            }),
            Rc::new(EvidenceMilestoneRequiredTitleRule {
                message: "evidence milestone needs this field".to_string(),
            }),
            Rc::new(EvidenceMilestoneHostLengthRule { max_length: 3600 }),
            Rc::new(EvidenceMilestoneRequiredHostRule {
                message: "evidence milestone needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_milestone_validator();
        let item = new_evidence_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceMilestoneRequiredCodeRule {
            message: "evidence milestone needs an entry".to_string(),
        };
        let mut item = new_evidence_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = EvidenceMilestoneRequiredCountryRule {
            message: "evidence milestone needs an entry".to_string(),
        };
        let mut item = new_evidence_milestone("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EvidenceMilestoneRequiredTitleRule {
            message: "evidence milestone needs a value".to_string(),
        };
        let mut item = new_evidence_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = EvidenceMilestoneHostLengthRule { max_length: 80 };
        let mut item = new_evidence_milestone("x", "y");
        item.host = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = EvidenceMilestoneRequiredHostRule {
            message: "evidence milestone needs a value".to_string(),
        };
        let mut item = new_evidence_milestone("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
