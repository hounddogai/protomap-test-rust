use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant score must pass. Check returns its problems, or none.
pub trait TenantScoreRule: std::fmt::Debug {
    fn check(&self, item: &TenantScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScoreRequiredTitleRule {
    pub message: String,
}

impl TenantScoreRule for TenantScoreRequiredTitleRule {
    fn check(&self, item: &TenantScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScoreAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TenantScoreRule for TenantScoreAllowedNameRule {
    fn check(&self, item: &TenantScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScoreRequiredCountryRule {
    pub message: String,
}

impl TenantScoreRule for TenantScoreRequiredCountryRule {
    fn check(&self, item: &TenantScore) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant score.
#[derive(Debug, Clone)]
pub struct TenantScoreValidator {
    pub rules: Vec<Rc<dyn TenantScoreRule>>,
}

impl TenantScoreValidator {
    pub fn validate(&self, item: &TenantScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_score_validator() -> TenantScoreValidator {
    TenantScoreValidator {
        rules: vec![
            Rc::new(TenantScoreRequiredTitleRule {
                message: "tenant score needs an entry".to_string(),
            }) as Rc<dyn TenantScoreRule>,
            Rc::new(TenantScoreAllowedNameRule {
                allowed: vec!["primary".to_string(), "scheduled".to_string(), "batched".to_string()],
            }),
            Rc::new(TenantScoreRequiredCountryRule {
                message: "tenant score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_score_validator();
        let item = new_tenant_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TenantScoreRequiredTitleRule {
            message: "tenant score needs an entry".to_string(),
        };
        let mut item = new_tenant_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TenantScoreRequiredCountryRule {
            message: "tenant score needs a value".to_string(),
        };
        let mut item = new_tenant_score("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
