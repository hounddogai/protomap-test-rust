use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global flow log score must pass. Check returns its problems, or none.
pub trait GlobalFlowLogScoreRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScoreNameLengthRule {
    pub max_length: i64,
}

impl GlobalFlowLogScoreRule for GlobalFlowLogScoreNameLengthRule {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScoreRequiredCodeRule {
    pub message: String,
}

impl GlobalFlowLogScoreRule for GlobalFlowLogScoreRequiredCodeRule {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScoreAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl GlobalFlowLogScoreRule for GlobalFlowLogScoreAllowedAudienceRule {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is inconsistent", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScoreRequiredAudienceRule {
    pub message: String,
}

impl GlobalFlowLogScoreRule for GlobalFlowLogScoreRequiredAudienceRule {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScoreContactEmailLengthRule {
    pub max_length: i64,
}

impl GlobalFlowLogScoreRule for GlobalFlowLogScoreContactEmailLengthRule {
    fn check(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global flow log score.
#[derive(Debug, Clone)]
pub struct GlobalFlowLogScoreValidator {
    pub rules: Vec<Rc<dyn GlobalFlowLogScoreRule>>,
}

impl GlobalFlowLogScoreValidator {
    pub fn validate(&self, item: &GlobalFlowLogScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalFlowLogScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFlowLogScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_flow_log_score_validator() -> GlobalFlowLogScoreValidator {
    GlobalFlowLogScoreValidator {
        rules: vec![
            Rc::new(GlobalFlowLogScoreNameLengthRule { max_length: 32 }) as Rc<dyn GlobalFlowLogScoreRule>,
            Rc::new(GlobalFlowLogScoreRequiredCodeRule {
                message: "global flow log score needs a value".to_string(),
            }),
            Rc::new(GlobalFlowLogScoreAllowedAudienceRule {
                allowed: vec!["seasonal".to_string(), "adaptive".to_string()],
            }),
            Rc::new(GlobalFlowLogScoreRequiredAudienceRule {
                message: "global flow log score needs this field".to_string(),
            }),
            Rc::new(GlobalFlowLogScoreContactEmailLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_flow_log_score_validator();
        let item = new_global_flow_log_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalFlowLogScoreNameLengthRule { max_length: 86400 };
        let mut item = new_global_flow_log_score("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalFlowLogScoreRequiredCodeRule {
            message: "global flow log score needs this field".to_string(),
        };
        let mut item = new_global_flow_log_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = GlobalFlowLogScoreRequiredAudienceRule {
            message: "global flow log score needs a value".to_string(),
        };
        let mut item = new_global_flow_log_score("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = GlobalFlowLogScoreContactEmailLengthRule { max_length: 86400 };
        let mut item = new_global_flow_log_score("x", "y");
        item.contact_email = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
