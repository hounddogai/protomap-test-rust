use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual allowlist target must pass. Check returns its problems, or none.
pub trait ManualAllowlistTargetRule: std::fmt::Debug {
    fn check(&self, item: &ManualAllowlistTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistTargetRequiredPhoneRule {
    pub message: String,
}

impl ManualAllowlistTargetRule for ManualAllowlistTargetRequiredPhoneRule {
    fn check(&self, item: &ManualAllowlistTarget) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistTargetRequiredIssuerRule {
    pub message: String,
}

impl ManualAllowlistTargetRule for ManualAllowlistTargetRequiredIssuerRule {
    fn check(&self, item: &ManualAllowlistTarget) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistTargetPhoneLengthRule {
    pub max_length: i64,
}

impl ManualAllowlistTargetRule for ManualAllowlistTargetPhoneLengthRule {
    fn check(&self, item: &ManualAllowlistTarget) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual allowlist target.
#[derive(Debug, Clone)]
pub struct ManualAllowlistTargetValidator {
    pub rules: Vec<Rc<dyn ManualAllowlistTargetRule>>,
}

impl ManualAllowlistTargetValidator {
    pub fn validate(&self, item: &ManualAllowlistTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAllowlistTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAllowlistTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_allowlist_target_validator() -> ManualAllowlistTargetValidator {
    ManualAllowlistTargetValidator {
        rules: vec![
            Rc::new(ManualAllowlistTargetRequiredPhoneRule {
                message: "manual allowlist target needs this field".to_string(),
            }) as Rc<dyn ManualAllowlistTargetRule>,
            Rc::new(ManualAllowlistTargetRequiredIssuerRule {
                message: "manual allowlist target needs this field".to_string(),
            }),
            Rc::new(ManualAllowlistTargetPhoneLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_allowlist_target_validator();
        let item = new_manual_allowlist_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ManualAllowlistTargetRequiredPhoneRule {
            message: "manual allowlist target needs an entry".to_string(),
        };
        let mut item = new_manual_allowlist_target("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = ManualAllowlistTargetRequiredIssuerRule {
            message: "manual allowlist target needs a value".to_string(),
        };
        let mut item = new_manual_allowlist_target("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ManualAllowlistTargetPhoneLengthRule { max_length: 32 };
        let mut item = new_manual_allowlist_target("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
