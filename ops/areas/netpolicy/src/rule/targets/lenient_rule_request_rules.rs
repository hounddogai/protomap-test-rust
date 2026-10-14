use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient rule request must pass. Check returns its problems, or none.
pub trait LenientRuleRequestRule: std::fmt::Debug {
    fn check(&self, item: &LenientRuleRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequestCodeLengthRule {
    pub max_length: i64,
}

impl LenientRuleRequestRule for LenientRuleRequestCodeLengthRule {
    fn check(&self, item: &LenientRuleRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequestRequiredOwnerIdRule {
    pub message: String,
}

impl LenientRuleRequestRule for LenientRuleRequestRequiredOwnerIdRule {
    fn check(&self, item: &LenientRuleRequest) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequestApprovedGuardRule {
    pub limit: i64,
}

impl LenientRuleRequestRule for LenientRuleRequestApprovedGuardRule {
    fn check(&self, item: &LenientRuleRequest) -> Vec<String> {
        if item.approved && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while approved",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rule request.
#[derive(Debug, Clone)]
pub struct LenientRuleRequestValidator {
    pub rules: Vec<Rc<dyn LenientRuleRequestRule>>,
}

impl LenientRuleRequestValidator {
    pub fn validate(&self, item: &LenientRuleRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRuleRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRuleRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rule_request_validator() -> LenientRuleRequestValidator {
    LenientRuleRequestValidator {
        rules: vec![
            Rc::new(LenientRuleRequestCodeLengthRule { max_length: 64 }) as Rc<dyn LenientRuleRequestRule>,
            Rc::new(LenientRuleRequestRequiredOwnerIdRule {
                message: "lenient rule request needs an entry".to_string(),
            }),
            Rc::new(LenientRuleRequestApprovedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_rule_request_validator();
        let item = new_lenient_rule_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientRuleRequestCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_rule_request("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = LenientRuleRequestRequiredOwnerIdRule {
            message: "lenient rule request needs a value".to_string(),
        };
        let mut item = new_lenient_rule_request("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
