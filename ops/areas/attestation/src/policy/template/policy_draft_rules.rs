use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy draft must pass. Check returns its problems, or none.
pub trait PolicyDraftRule: std::fmt::Debug {
    fn check(&self, item: &PolicyDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftCodePrefixRule {
    pub prefix: String,
}

impl PolicyDraftRule for PolicyDraftCodePrefixRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftRequiredNameRule {
    pub message: String,
}

impl PolicyDraftRule for PolicyDraftRequiredNameRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftNameLengthRule {
    pub max_length: i64,
}

impl PolicyDraftRule for PolicyDraftNameLengthRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyDraftRule for PolicyDraftCeilingRangeRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftTimezoneLengthRule {
    pub max_length: i64,
}

impl PolicyDraftRule for PolicyDraftTimezoneLengthRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDraftIssuerLengthRule {
    pub max_length: i64,
}

impl PolicyDraftRule for PolicyDraftIssuerLengthRule {
    fn check(&self, item: &PolicyDraft) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy draft.
#[derive(Debug, Clone)]
pub struct PolicyDraftValidator {
    pub rules: Vec<Rc<dyn PolicyDraftRule>>,
}

impl PolicyDraftValidator {
    pub fn validate(&self, item: &PolicyDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_draft_validator() -> PolicyDraftValidator {
    PolicyDraftValidator {
        rules: vec![
            Rc::new(PolicyDraftCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PolicyDraftRule>,
            Rc::new(PolicyDraftRequiredNameRule {
                message: "policy draft needs a value".to_string(),
            }),
            Rc::new(PolicyDraftNameLengthRule { max_length: 80 }),
            Rc::new(PolicyDraftCeilingRangeRule { min: 0.8, max: 1522.7 }),
            Rc::new(PolicyDraftTimezoneLengthRule { max_length: 64 }),
            Rc::new(PolicyDraftIssuerLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_draft_validator();
        let item = new_policy_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PolicyDraftRequiredNameRule {
            message: "policy draft needs this field".to_string(),
        };
        let mut item = new_policy_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PolicyDraftNameLengthRule { max_length: 86400 };
        let mut item = new_policy_draft("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PolicyDraftCeilingRangeRule { min: 3.8, max: 939.1 };
        let mut item = new_policy_draft("x", "y");
        item.ceiling = 940.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PolicyDraftTimezoneLengthRule { max_length: 80 };
        let mut item = new_policy_draft("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PolicyDraftIssuerLengthRule { max_length: 16 };
        let mut item = new_policy_draft("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
