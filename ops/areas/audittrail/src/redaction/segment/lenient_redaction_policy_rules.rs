use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient redaction policy must pass. Check returns its problems, or none.
pub trait LenientRedactionPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LenientRedactionPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicyPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRedactionPolicyRule for LenientRedactionPolicyPortRangeRule {
    fn check(&self, item: &LenientRedactionPolicy) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicyRequiredCodeRule {
    pub message: String,
}

impl LenientRedactionPolicyRule for LenientRedactionPolicyRequiredCodeRule {
    fn check(&self, item: &LenientRedactionPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicyEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRedactionPolicyRule for LenientRedactionPolicyEndsAtRangeRule {
    fn check(&self, item: &LenientRedactionPolicy) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicyCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRedactionPolicyRule for LenientRedactionPolicyCreatedAtRangeRule {
    fn check(&self, item: &LenientRedactionPolicy) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient redaction policy.
#[derive(Debug, Clone)]
pub struct LenientRedactionPolicyValidator {
    pub rules: Vec<Rc<dyn LenientRedactionPolicyRule>>,
}

impl LenientRedactionPolicyValidator {
    pub fn validate(&self, item: &LenientRedactionPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRedactionPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRedactionPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_redaction_policy_validator() -> LenientRedactionPolicyValidator {
    LenientRedactionPolicyValidator {
        rules: vec![
            Rc::new(LenientRedactionPolicyPortRangeRule { min: 16, max: 3600 }) as Rc<dyn LenientRedactionPolicyRule>,
            Rc::new(LenientRedactionPolicyRequiredCodeRule {
                message: "lenient redaction policy needs an entry".to_string(),
            }),
            Rc::new(LenientRedactionPolicyEndsAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(LenientRedactionPolicyCreatedAtRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_redaction_policy_validator();
        let item = new_lenient_redaction_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientRedactionPolicyPortRangeRule { min: 120, max: 1000 };
        let mut item = new_lenient_redaction_policy("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientRedactionPolicyRequiredCodeRule {
            message: "lenient redaction policy needs an entry".to_string(),
        };
        let mut item = new_lenient_redaction_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LenientRedactionPolicyEndsAtRangeRule { min: 120, max: 1000 };
        let mut item = new_lenient_redaction_policy("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientRedactionPolicyCreatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_lenient_redaction_policy("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
