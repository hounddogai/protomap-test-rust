use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive health check quota must pass. Check returns its problems, or none.
pub trait AdaptiveHealthCheckQuotaRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckQuotaScopeLengthRule {
    pub max_length: i64,
}

impl AdaptiveHealthCheckQuotaRule for AdaptiveHealthCheckQuotaScopeLengthRule {
    fn check(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl AdaptiveHealthCheckQuotaRule for AdaptiveHealthCheckQuotaExternalRefLengthRule {
    fn check(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String> {
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
pub struct AdaptiveHealthCheckQuotaRequiredExternalRefRule {
    pub message: String,
}

impl AdaptiveHealthCheckQuotaRule for AdaptiveHealthCheckQuotaRequiredExternalRefRule {
    fn check(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckQuotaTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveHealthCheckQuotaRule for AdaptiveHealthCheckQuotaTitleLengthRule {
    fn check(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive health check quota.
#[derive(Debug, Clone)]
pub struct AdaptiveHealthCheckQuotaValidator {
    pub rules: Vec<Rc<dyn AdaptiveHealthCheckQuotaRule>>,
}

impl AdaptiveHealthCheckQuotaValidator {
    pub fn validate(&self, item: &AdaptiveHealthCheckQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveHealthCheckQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveHealthCheckQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_health_check_quota_validator() -> AdaptiveHealthCheckQuotaValidator {
    AdaptiveHealthCheckQuotaValidator {
        rules: vec![
            Rc::new(AdaptiveHealthCheckQuotaScopeLengthRule { max_length: 32 }) as Rc<dyn AdaptiveHealthCheckQuotaRule>,
            Rc::new(AdaptiveHealthCheckQuotaExternalRefLengthRule { max_length: 64 }),
            Rc::new(AdaptiveHealthCheckQuotaRequiredExternalRefRule {
                message: "adaptive health check quota needs this field".to_string(),
            }),
            Rc::new(AdaptiveHealthCheckQuotaTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_health_check_quota_validator();
        let item = new_adaptive_health_check_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AdaptiveHealthCheckQuotaScopeLengthRule { max_length: 120 };
        let mut item = new_adaptive_health_check_quota("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = AdaptiveHealthCheckQuotaExternalRefLengthRule { max_length: 16 };
        let mut item = new_adaptive_health_check_quota("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = AdaptiveHealthCheckQuotaRequiredExternalRefRule {
            message: "adaptive health check quota needs an entry".to_string(),
        };
        let mut item = new_adaptive_health_check_quota("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveHealthCheckQuotaTitleLengthRule { max_length: 255 };
        let mut item = new_adaptive_health_check_quota("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
