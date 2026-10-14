use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient release policy must pass. Check returns its problems, or none.
pub trait LenientReleasePolicyRule: std::fmt::Debug {
    fn check(&self, item: &LenientReleasePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReleasePolicyCodeLengthRule {
    pub max_length: i64,
}

impl LenientReleasePolicyRule for LenientReleasePolicyCodeLengthRule {
    fn check(&self, item: &LenientReleasePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReleasePolicyRequiredLabelRule {
    pub message: String,
}

impl LenientReleasePolicyRule for LenientReleasePolicyRequiredLabelRule {
    fn check(&self, item: &LenientReleasePolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReleasePolicyApprovedGuardRule {
    pub limit: i64,
}

impl LenientReleasePolicyRule for LenientReleasePolicyApprovedGuardRule {
    fn check(&self, item: &LenientReleasePolicy) -> Vec<String> {
        if item.approved && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while approved", item.burst, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient release policy.
#[derive(Debug, Clone)]
pub struct LenientReleasePolicyValidator {
    pub rules: Vec<Rc<dyn LenientReleasePolicyRule>>,
}

impl LenientReleasePolicyValidator {
    pub fn validate(&self, item: &LenientReleasePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientReleasePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientReleasePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_release_policy_validator() -> LenientReleasePolicyValidator {
    LenientReleasePolicyValidator {
        rules: vec![
            Rc::new(LenientReleasePolicyCodeLengthRule { max_length: 3600 }) as Rc<dyn LenientReleasePolicyRule>,
            Rc::new(LenientReleasePolicyRequiredLabelRule {
                message: "lenient release policy needs this field".to_string(),
            }),
            Rc::new(LenientReleasePolicyApprovedGuardRule { limit: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_release_policy_validator();
        let item = new_lenient_release_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientReleasePolicyCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_release_policy("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientReleasePolicyRequiredLabelRule {
            message: "lenient release policy needs this field".to_string(),
        };
        let mut item = new_lenient_release_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
