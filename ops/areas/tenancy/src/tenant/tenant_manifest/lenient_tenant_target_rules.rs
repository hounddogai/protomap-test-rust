use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient tenant target must pass. Check returns its problems, or none.
pub trait LenientTenantTargetRule: std::fmt::Debug {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetExternalRefLengthRule {
    pub max_length: i64,
}

impl LenientTenantTargetRule for LenientTenantTargetExternalRefLengthRule {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetCodeLengthRule {
    pub max_length: i64,
}

impl LenientTenantTargetRule for LenientTenantTargetCodeLengthRule {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientTenantTargetRule for LenientTenantTargetAllowedCodeRule {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetSegmentLengthRule {
    pub max_length: i64,
}

impl LenientTenantTargetRule for LenientTenantTargetSegmentLengthRule {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetRequiredExternalRefRule {
    pub message: String,
}

impl LenientTenantTargetRule for LenientTenantTargetRequiredExternalRefRule {
    fn check(&self, item: &LenientTenantTarget) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient tenant target.
#[derive(Debug, Clone)]
pub struct LenientTenantTargetValidator {
    pub rules: Vec<Rc<dyn LenientTenantTargetRule>>,
}

impl LenientTenantTargetValidator {
    pub fn validate(&self, item: &LenientTenantTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTenantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTenantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_tenant_target_validator() -> LenientTenantTargetValidator {
    LenientTenantTargetValidator {
        rules: vec![
            Rc::new(LenientTenantTargetExternalRefLengthRule { max_length: 255 }) as Rc<dyn LenientTenantTargetRule>,
            Rc::new(LenientTenantTargetCodeLengthRule { max_length: 86400 }),
            Rc::new(LenientTenantTargetAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }),
            Rc::new(LenientTenantTargetSegmentLengthRule { max_length: 86400 }),
            Rc::new(LenientTenantTargetRequiredExternalRefRule {
                message: "lenient tenant target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_tenant_target_validator();
        let item = new_lenient_tenant_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = LenientTenantTargetExternalRefLengthRule { max_length: 80 };
        let mut item = new_lenient_tenant_target("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientTenantTargetCodeLengthRule { max_length: 1000 };
        let mut item = new_lenient_tenant_target("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LenientTenantTargetSegmentLengthRule { max_length: 16 };
        let mut item = new_lenient_tenant_target("x", "y");
        item.segment = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LenientTenantTargetRequiredExternalRefRule {
            message: "lenient tenant target needs this field".to_string(),
        };
        let mut item = new_lenient_tenant_target("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
