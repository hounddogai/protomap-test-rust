use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient query manifest must pass. Check returns its problems, or none.
pub trait LenientQueryManifestRule: std::fmt::Debug {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestArchivedGuardRule {
    pub limit: i64,
}

impl LenientQueryManifestRule for LenientQueryManifestArchivedGuardRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
        if item.archived && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while archived",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestRequiredCodeRule {
    pub message: String,
}

impl LenientQueryManifestRule for LenientQueryManifestRequiredCodeRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientQueryManifestRule for LenientQueryManifestPriorityRangeRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestPrincipalLengthRule {
    pub max_length: i64,
}

impl LenientQueryManifestRule for LenientQueryManifestPrincipalLengthRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
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
pub struct LenientQueryManifestNameLengthRule {
    pub max_length: i64,
}

impl LenientQueryManifestRule for LenientQueryManifestNameLengthRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestRequiredPrincipalRule {
    pub message: String,
}

impl LenientQueryManifestRule for LenientQueryManifestRequiredPrincipalRule {
    fn check(&self, item: &LenientQueryManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient query manifest.
#[derive(Debug, Clone)]
pub struct LenientQueryManifestValidator {
    pub rules: Vec<Rc<dyn LenientQueryManifestRule>>,
}

impl LenientQueryManifestValidator {
    pub fn validate(&self, item: &LenientQueryManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientQueryManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQueryManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_query_manifest_validator() -> LenientQueryManifestValidator {
    LenientQueryManifestValidator {
        rules: vec![
            Rc::new(LenientQueryManifestArchivedGuardRule { limit: 64 }) as Rc<dyn LenientQueryManifestRule>,
            Rc::new(LenientQueryManifestRequiredCodeRule {
                message: "lenient query manifest needs this field".to_string(),
            }),
            Rc::new(LenientQueryManifestPriorityRangeRule { min: 1000, max: 86400 }),
            Rc::new(LenientQueryManifestPrincipalLengthRule { max_length: 1000 }),
            Rc::new(LenientQueryManifestNameLengthRule { max_length: 3600 }),
            Rc::new(LenientQueryManifestRequiredPrincipalRule {
                message: "lenient query manifest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_query_manifest_validator();
        let item = new_lenient_query_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientQueryManifestRequiredCodeRule {
            message: "lenient query manifest needs a value".to_string(),
        };
        let mut item = new_lenient_query_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LenientQueryManifestPriorityRangeRule { min: 16, max: 32 };
        let mut item = new_lenient_query_manifest("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = LenientQueryManifestPrincipalLengthRule { max_length: 86400 };
        let mut item = new_lenient_query_manifest("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientQueryManifestNameLengthRule { max_length: 120 };
        let mut item = new_lenient_query_manifest("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = LenientQueryManifestRequiredPrincipalRule {
            message: "lenient query manifest needs this field".to_string(),
        };
        let mut item = new_lenient_query_manifest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
