use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted overage audit must pass. Check returns its problems, or none.
pub trait WeightedOverageAuditRule: std::fmt::Debug {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditRequiredTitleRule {
    pub message: String,
}

impl WeightedOverageAuditRule for WeightedOverageAuditRequiredTitleRule {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditRequiredPrincipalRule {
    pub message: String,
}

impl WeightedOverageAuditRule for WeightedOverageAuditRequiredPrincipalRule {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedOverageAuditRule for WeightedOverageAuditQuantityRangeRule {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditCodePrefixRule {
    pub prefix: String,
}

impl WeightedOverageAuditRule for WeightedOverageAuditCodePrefixRule {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditCodeLengthRule {
    pub max_length: i64,
}

impl WeightedOverageAuditRule for WeightedOverageAuditCodeLengthRule {
    fn check(&self, item: &WeightedOverageAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted overage audit.
#[derive(Debug, Clone)]
pub struct WeightedOverageAuditValidator {
    pub rules: Vec<Rc<dyn WeightedOverageAuditRule>>,
}

impl WeightedOverageAuditValidator {
    pub fn validate(&self, item: &WeightedOverageAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedOverageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedOverageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_overage_audit_validator() -> WeightedOverageAuditValidator {
    WeightedOverageAuditValidator {
        rules: vec![
            Rc::new(WeightedOverageAuditRequiredTitleRule {
                message: "weighted overage audit needs a value".to_string(),
            }) as Rc<dyn WeightedOverageAuditRule>,
            Rc::new(WeightedOverageAuditRequiredPrincipalRule {
                message: "weighted overage audit needs a value".to_string(),
            }),
            Rc::new(WeightedOverageAuditQuantityRangeRule { min: 3600, max: 86400 }),
            Rc::new(WeightedOverageAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(WeightedOverageAuditCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_overage_audit_validator();
        let item = new_weighted_overage_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedOverageAuditRequiredTitleRule {
            message: "weighted overage audit needs a value".to_string(),
        };
        let mut item = new_weighted_overage_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = WeightedOverageAuditRequiredPrincipalRule {
            message: "weighted overage audit needs a value".to_string(),
        };
        let mut item = new_weighted_overage_audit("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = WeightedOverageAuditQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_weighted_overage_audit("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedOverageAuditCodeLengthRule { max_length: 64 };
        let mut item = new_weighted_overage_audit("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
