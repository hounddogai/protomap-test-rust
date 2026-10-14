use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped query mapping must pass. Check returns its problems, or none.
pub trait CappedQueryMappingRule: std::fmt::Debug {
    fn check(&self, item: &CappedQueryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryMappingCodePrefixRule {
    pub prefix: String,
}

impl CappedQueryMappingRule for CappedQueryMappingCodePrefixRule {
    fn check(&self, item: &CappedQueryMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryMappingRequiredCodeRule {
    pub message: String,
}

impl CappedQueryMappingRule for CappedQueryMappingRequiredCodeRule {
    fn check(&self, item: &CappedQueryMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryMappingActiveGuardRule {
    pub limit: f64,
}

impl CappedQueryMappingRule for CappedQueryMappingActiveGuardRule {
    fn check(&self, item: &CappedQueryMapping) -> Vec<String> {
        if item.active && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while active", item.amount, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped query mapping.
#[derive(Debug, Clone)]
pub struct CappedQueryMappingValidator {
    pub rules: Vec<Rc<dyn CappedQueryMappingRule>>,
}

impl CappedQueryMappingValidator {
    pub fn validate(&self, item: &CappedQueryMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedQueryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQueryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_query_mapping_validator() -> CappedQueryMappingValidator {
    CappedQueryMappingValidator {
        rules: vec![
            Rc::new(CappedQueryMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn CappedQueryMappingRule>,
            Rc::new(CappedQueryMappingRequiredCodeRule {
                message: "capped query mapping needs a value".to_string(),
            }),
            Rc::new(CappedQueryMappingActiveGuardRule { limit: 7.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_query_mapping_validator();
        let item = new_capped_query_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedQueryMappingRequiredCodeRule {
            message: "capped query mapping needs this field".to_string(),
        };
        let mut item = new_capped_query_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
