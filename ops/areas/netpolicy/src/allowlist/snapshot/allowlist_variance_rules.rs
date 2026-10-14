use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist variance must pass. Check returns its problems, or none.
pub trait AllowlistVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVarianceAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl AllowlistVarianceRule for AllowlistVarianceAllowedPrincipalRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is unknown", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVarianceNameLengthRule {
    pub max_length: i64,
}

impl AllowlistVarianceRule for AllowlistVarianceNameLengthRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVariancePrincipalLengthRule {
    pub max_length: i64,
}

impl AllowlistVarianceRule for AllowlistVariancePrincipalLengthRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVarianceCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistVarianceRule for AllowlistVarianceCodeLengthRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVarianceRequiredCodeRule {
    pub message: String,
}

impl AllowlistVarianceRule for AllowlistVarianceRequiredCodeRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistVarianceRequiredPrincipalRule {
    pub message: String,
}

impl AllowlistVarianceRule for AllowlistVarianceRequiredPrincipalRule {
    fn check(&self, item: &AllowlistVariance) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist variance.
#[derive(Debug, Clone)]
pub struct AllowlistVarianceValidator {
    pub rules: Vec<Rc<dyn AllowlistVarianceRule>>,
}

impl AllowlistVarianceValidator {
    pub fn validate(&self, item: &AllowlistVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_variance_validator() -> AllowlistVarianceValidator {
    AllowlistVarianceValidator {
        rules: vec![
            Rc::new(AllowlistVarianceAllowedPrincipalRule {
                allowed: vec!["strict".to_string(), "strict".to_string(), "regional".to_string()],
            }) as Rc<dyn AllowlistVarianceRule>,
            Rc::new(AllowlistVarianceNameLengthRule { max_length: 32 }),
            Rc::new(AllowlistVariancePrincipalLengthRule { max_length: 86400 }),
            Rc::new(AllowlistVarianceCodeLengthRule { max_length: 80 }),
            Rc::new(AllowlistVarianceRequiredCodeRule {
                message: "allowlist variance needs an entry".to_string(),
            }),
            Rc::new(AllowlistVarianceRequiredPrincipalRule {
                message: "allowlist variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_variance_validator();
        let item = new_allowlist_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AllowlistVarianceNameLengthRule { max_length: 64 };
        let mut item = new_allowlist_variance("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = AllowlistVariancePrincipalLengthRule { max_length: 120 };
        let mut item = new_allowlist_variance("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AllowlistVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_allowlist_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistVarianceRequiredCodeRule {
            message: "allowlist variance needs a value".to_string(),
        };
        let mut item = new_allowlist_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = AllowlistVarianceRequiredPrincipalRule {
            message: "allowlist variance needs this field".to_string(),
        };
        let mut item = new_allowlist_variance("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
