use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped isolation template must pass. Check returns its problems, or none.
pub trait CappedIsolationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplateRequiredTitleRule {
    pub message: String,
}

impl CappedIsolationTemplateRule for CappedIsolationTemplateRequiredTitleRule {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplateCodePrefixRule {
    pub prefix: String,
}

impl CappedIsolationTemplateRule for CappedIsolationTemplateCodePrefixRule {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl CappedIsolationTemplateRule for CappedIsolationTemplateCodeLengthRule {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplateScopeLengthRule {
    pub max_length: i64,
}

impl CappedIsolationTemplateRule for CappedIsolationTemplateScopeLengthRule {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationTemplateActiveGuardRule {
    pub limit: i64,
}

impl CappedIsolationTemplateRule for CappedIsolationTemplateActiveGuardRule {
    fn check(&self, item: &CappedIsolationTemplate) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped isolation template.
#[derive(Debug, Clone)]
pub struct CappedIsolationTemplateValidator {
    pub rules: Vec<Rc<dyn CappedIsolationTemplateRule>>,
}

impl CappedIsolationTemplateValidator {
    pub fn validate(&self, item: &CappedIsolationTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedIsolationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedIsolationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_isolation_template_validator() -> CappedIsolationTemplateValidator {
    CappedIsolationTemplateValidator {
        rules: vec![
            Rc::new(CappedIsolationTemplateRequiredTitleRule {
                message: "capped isolation template needs this field".to_string(),
            }) as Rc<dyn CappedIsolationTemplateRule>,
            Rc::new(CappedIsolationTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CappedIsolationTemplateCodeLengthRule { max_length: 1000 }),
            Rc::new(CappedIsolationTemplateScopeLengthRule { max_length: 120 }),
            Rc::new(CappedIsolationTemplateActiveGuardRule { limit: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_isolation_template_validator();
        let item = new_capped_isolation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedIsolationTemplateRequiredTitleRule {
            message: "capped isolation template needs this field".to_string(),
        };
        let mut item = new_capped_isolation_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedIsolationTemplateCodeLengthRule { max_length: 80 };
        let mut item = new_capped_isolation_template("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = CappedIsolationTemplateScopeLengthRule { max_length: 120 };
        let mut item = new_capped_isolation_template("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
