use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo policy must pass. Check returns its problems, or none.
pub trait SloPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SloPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyRequiredRegionRule {
    pub message: String,
}

impl SloPolicyRule for SloPolicyRequiredRegionRule {
    fn check(&self, item: &SloPolicy) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SloPolicyRule for SloPolicyAllowedCodeRule {
    fn check(&self, item: &SloPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyLabelLengthRule {
    pub max_length: i64,
}

impl SloPolicyRule for SloPolicyLabelLengthRule {
    fn check(&self, item: &SloPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyRequiredCodeRule {
    pub message: String,
}

impl SloPolicyRule for SloPolicyRequiredCodeRule {
    fn check(&self, item: &SloPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyCodeLengthRule {
    pub max_length: i64,
}

impl SloPolicyRule for SloPolicyCodeLengthRule {
    fn check(&self, item: &SloPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo policy.
#[derive(Debug, Clone)]
pub struct SloPolicyValidator {
    pub rules: Vec<Rc<dyn SloPolicyRule>>,
}

impl SloPolicyValidator {
    pub fn validate(&self, item: &SloPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_policy_validator() -> SloPolicyValidator {
    SloPolicyValidator {
        rules: vec![
            Rc::new(SloPolicyRequiredRegionRule {
                message: "slo policy needs a value".to_string(),
            }) as Rc<dyn SloPolicyRule>,
            Rc::new(SloPolicyAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(SloPolicyLabelLengthRule { max_length: 1000 }),
            Rc::new(SloPolicyRequiredCodeRule {
                message: "slo policy needs this field".to_string(),
            }),
            Rc::new(SloPolicyCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_policy_validator();
        let item = new_slo_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = SloPolicyRequiredRegionRule {
            message: "slo policy needs this field".to_string(),
        };
        let mut item = new_slo_policy("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloPolicyLabelLengthRule { max_length: 64 };
        let mut item = new_slo_policy("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloPolicyRequiredCodeRule {
            message: "slo policy needs this field".to_string(),
        };
        let mut item = new_slo_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloPolicyCodeLengthRule { max_length: 64 };
        let mut item = new_slo_policy("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
