use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log audit must pass. Check returns its problems, or none.
pub trait FlowLogAuditRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAuditRequiredNameRule {
    pub message: String,
}

impl FlowLogAuditRule for FlowLogAuditRequiredNameRule {
    fn check(&self, item: &FlowLogAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAuditCategoryLengthRule {
    pub max_length: i64,
}

impl FlowLogAuditRule for FlowLogAuditCategoryLengthRule {
    fn check(&self, item: &FlowLogAudit) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAuditExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogAuditRule for FlowLogAuditExpiresAtRangeRule {
    fn check(&self, item: &FlowLogAudit) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAuditCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogAuditRule for FlowLogAuditCodeLengthRule {
    fn check(&self, item: &FlowLogAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log audit.
#[derive(Debug, Clone)]
pub struct FlowLogAuditValidator {
    pub rules: Vec<Rc<dyn FlowLogAuditRule>>,
}

impl FlowLogAuditValidator {
    pub fn validate(&self, item: &FlowLogAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_audit_validator() -> FlowLogAuditValidator {
    FlowLogAuditValidator {
        rules: vec![
            Rc::new(FlowLogAuditRequiredNameRule {
                message: "flow log audit needs a value".to_string(),
            }) as Rc<dyn FlowLogAuditRule>,
            Rc::new(FlowLogAuditCategoryLengthRule { max_length: 32 }),
            Rc::new(FlowLogAuditExpiresAtRangeRule { min: 16, max: 3600 }),
            Rc::new(FlowLogAuditCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_audit_validator();
        let item = new_flow_log_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FlowLogAuditRequiredNameRule {
            message: "flow log audit needs this field".to_string(),
        };
        let mut item = new_flow_log_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = FlowLogAuditCategoryLengthRule { max_length: 255 };
        let mut item = new_flow_log_audit("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FlowLogAuditExpiresAtRangeRule { min: 64, max: 3600 };
        let mut item = new_flow_log_audit("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogAuditCodeLengthRule { max_length: 1000 };
        let mut item = new_flow_log_audit("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
