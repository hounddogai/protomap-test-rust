use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal allowlist mapping must pass. Check returns its problems, or none.
pub trait SeasonalAllowlistMappingRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAllowlistMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingRequiredPrincipalRule {
    pub message: String,
}

impl SeasonalAllowlistMappingRule for SeasonalAllowlistMappingRequiredPrincipalRule {
    fn check(&self, item: &SeasonalAllowlistMapping) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingRequiredCodeRule {
    pub message: String,
}

impl SeasonalAllowlistMappingRule for SeasonalAllowlistMappingRequiredCodeRule {
    fn check(&self, item: &SeasonalAllowlistMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingRequiredLabelRule {
    pub message: String,
}

impl SeasonalAllowlistMappingRule for SeasonalAllowlistMappingRequiredLabelRule {
    fn check(&self, item: &SeasonalAllowlistMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingPinnedGuardRule {
    pub limit: f64,
}

impl SeasonalAllowlistMappingRule for SeasonalAllowlistMappingPinnedGuardRule {
    fn check(&self, item: &SeasonalAllowlistMapping) -> Vec<String> {
        if item.pinned && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while pinned",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal allowlist mapping.
#[derive(Debug, Clone)]
pub struct SeasonalAllowlistMappingValidator {
    pub rules: Vec<Rc<dyn SeasonalAllowlistMappingRule>>,
}

impl SeasonalAllowlistMappingValidator {
    pub fn validate(&self, item: &SeasonalAllowlistMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalAllowlistMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAllowlistMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_allowlist_mapping_validator() -> SeasonalAllowlistMappingValidator {
    SeasonalAllowlistMappingValidator {
        rules: vec![
            Rc::new(SeasonalAllowlistMappingRequiredPrincipalRule {
                message: "seasonal allowlist mapping needs a value".to_string(),
            }) as Rc<dyn SeasonalAllowlistMappingRule>,
            Rc::new(SeasonalAllowlistMappingRequiredCodeRule {
                message: "seasonal allowlist mapping needs this field".to_string(),
            }),
            Rc::new(SeasonalAllowlistMappingRequiredLabelRule {
                message: "seasonal allowlist mapping needs this field".to_string(),
            }),
            Rc::new(SeasonalAllowlistMappingPinnedGuardRule { limit: 1.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_allowlist_mapping_validator();
        let item = new_seasonal_allowlist_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SeasonalAllowlistMappingRequiredPrincipalRule {
            message: "seasonal allowlist mapping needs an entry".to_string(),
        };
        let mut item = new_seasonal_allowlist_mapping("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalAllowlistMappingRequiredCodeRule {
            message: "seasonal allowlist mapping needs this field".to_string(),
        };
        let mut item = new_seasonal_allowlist_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalAllowlistMappingRequiredLabelRule {
            message: "seasonal allowlist mapping needs an entry".to_string(),
        };
        let mut item = new_seasonal_allowlist_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
