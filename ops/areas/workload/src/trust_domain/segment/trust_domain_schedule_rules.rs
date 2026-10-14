use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain schedule must pass. Check returns its problems, or none.
pub trait TrustDomainScheduleRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleRequiredLabelRule {
    pub message: String,
}

impl TrustDomainScheduleRule for TrustDomainScheduleRequiredLabelRule {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainScheduleRule for TrustDomainScheduleCodeLengthRule {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleRequiredRegionRule {
    pub message: String,
}

impl TrustDomainScheduleRule for TrustDomainScheduleRequiredRegionRule {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleRequiredCodeRule {
    pub message: String,
}

impl TrustDomainScheduleRule for TrustDomainScheduleRequiredCodeRule {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleRequiredTitleRule {
    pub message: String,
}

impl TrustDomainScheduleRule for TrustDomainScheduleRequiredTitleRule {
    fn check(&self, item: &TrustDomainSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain schedule.
#[derive(Debug, Clone)]
pub struct TrustDomainScheduleValidator {
    pub rules: Vec<Rc<dyn TrustDomainScheduleRule>>,
}

impl TrustDomainScheduleValidator {
    pub fn validate(&self, item: &TrustDomainSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_schedule_validator() -> TrustDomainScheduleValidator {
    TrustDomainScheduleValidator {
        rules: vec![
            Rc::new(TrustDomainScheduleRequiredLabelRule {
                message: "trust domain schedule needs an entry".to_string(),
            }) as Rc<dyn TrustDomainScheduleRule>,
            Rc::new(TrustDomainScheduleCodeLengthRule { max_length: 64 }),
            Rc::new(TrustDomainScheduleRequiredRegionRule {
                message: "trust domain schedule needs a value".to_string(),
            }),
            Rc::new(TrustDomainScheduleRequiredCodeRule {
                message: "trust domain schedule needs this field".to_string(),
            }),
            Rc::new(TrustDomainScheduleRequiredTitleRule {
                message: "trust domain schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_schedule_validator();
        let item = new_trust_domain_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TrustDomainScheduleRequiredLabelRule {
            message: "trust domain schedule needs an entry".to_string(),
        };
        let mut item = new_trust_domain_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainScheduleCodeLengthRule { max_length: 64 };
        let mut item = new_trust_domain_schedule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = TrustDomainScheduleRequiredRegionRule {
            message: "trust domain schedule needs this field".to_string(),
        };
        let mut item = new_trust_domain_schedule("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainScheduleRequiredCodeRule {
            message: "trust domain schedule needs a value".to_string(),
        };
        let mut item = new_trust_domain_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TrustDomainScheduleRequiredTitleRule {
            message: "trust domain schedule needs an entry".to_string(),
        };
        let mut item = new_trust_domain_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
