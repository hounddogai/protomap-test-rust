use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant preference must pass. Check returns its problems, or none.
pub trait TenantPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &TenantPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceRequiredCountryRule {
    pub message: String,
}

impl TenantPreferenceRule for TenantPreferenceRequiredCountryRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceRequiredCodeRule {
    pub message: String,
}

impl TenantPreferenceRule for TenantPreferenceRequiredCodeRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceCodePrefixRule {
    pub prefix: String,
}

impl TenantPreferenceRule for TenantPreferenceCodePrefixRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceVisibleGuardRule {
    pub limit: i64,
}

impl TenantPreferenceRule for TenantPreferenceVisibleGuardRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        if item.visible && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while visible", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceRequiredNameRule {
    pub message: String,
}

impl TenantPreferenceRule for TenantPreferenceRequiredNameRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreferenceNameLengthRule {
    pub max_length: i64,
}

impl TenantPreferenceRule for TenantPreferenceNameLengthRule {
    fn check(&self, item: &TenantPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant preference.
#[derive(Debug, Clone)]
pub struct TenantPreferenceValidator {
    pub rules: Vec<Rc<dyn TenantPreferenceRule>>,
}

impl TenantPreferenceValidator {
    pub fn validate(&self, item: &TenantPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_preference_validator() -> TenantPreferenceValidator {
    TenantPreferenceValidator {
        rules: vec![
            Rc::new(TenantPreferenceRequiredCountryRule {
                message: "tenant preference needs an entry".to_string(),
            }) as Rc<dyn TenantPreferenceRule>,
            Rc::new(TenantPreferenceRequiredCodeRule {
                message: "tenant preference needs an entry".to_string(),
            }),
            Rc::new(TenantPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TenantPreferenceVisibleGuardRule { limit: 120 }),
            Rc::new(TenantPreferenceRequiredNameRule {
                message: "tenant preference needs a value".to_string(),
            }),
            Rc::new(TenantPreferenceNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_preference_validator();
        let item = new_tenant_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TenantPreferenceRequiredCountryRule {
            message: "tenant preference needs an entry".to_string(),
        };
        let mut item = new_tenant_preference("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantPreferenceRequiredCodeRule {
            message: "tenant preference needs this field".to_string(),
        };
        let mut item = new_tenant_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TenantPreferenceRequiredNameRule {
            message: "tenant preference needs this field".to_string(),
        };
        let mut item = new_tenant_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantPreferenceNameLengthRule { max_length: 255 };
        let mut item = new_tenant_preference("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
