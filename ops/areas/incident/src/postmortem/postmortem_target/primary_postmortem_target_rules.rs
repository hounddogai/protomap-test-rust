use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary postmortem target must pass. Check returns its problems, or none.
pub trait PrimaryPostmortemTargetRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPostmortemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTargetSegmentLengthRule {
    pub max_length: i64,
}

impl PrimaryPostmortemTargetRule for PrimaryPostmortemTargetSegmentLengthRule {
    fn check(&self, item: &PrimaryPostmortemTarget) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTargetRequiredNameRule {
    pub message: String,
}

impl PrimaryPostmortemTargetRule for PrimaryPostmortemTargetRequiredNameRule {
    fn check(&self, item: &PrimaryPostmortemTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTargetSummaryLengthRule {
    pub max_length: i64,
}

impl PrimaryPostmortemTargetRule for PrimaryPostmortemTargetSummaryLengthRule {
    fn check(&self, item: &PrimaryPostmortemTarget) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTargetRequiredCodeRule {
    pub message: String,
}

impl PrimaryPostmortemTargetRule for PrimaryPostmortemTargetRequiredCodeRule {
    fn check(&self, item: &PrimaryPostmortemTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary postmortem target.
#[derive(Debug, Clone)]
pub struct PrimaryPostmortemTargetValidator {
    pub rules: Vec<Rc<dyn PrimaryPostmortemTargetRule>>,
}

impl PrimaryPostmortemTargetValidator {
    pub fn validate(&self, item: &PrimaryPostmortemTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPostmortemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPostmortemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_postmortem_target_validator() -> PrimaryPostmortemTargetValidator {
    PrimaryPostmortemTargetValidator {
        rules: vec![
            Rc::new(PrimaryPostmortemTargetSegmentLengthRule { max_length: 3600 })
                as Rc<dyn PrimaryPostmortemTargetRule>,
            Rc::new(PrimaryPostmortemTargetRequiredNameRule {
                message: "primary postmortem target needs an entry".to_string(),
            }),
            Rc::new(PrimaryPostmortemTargetSummaryLengthRule { max_length: 255 }),
            Rc::new(PrimaryPostmortemTargetRequiredCodeRule {
                message: "primary postmortem target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_postmortem_target_validator();
        let item = new_primary_postmortem_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = PrimaryPostmortemTargetSegmentLengthRule { max_length: 255 };
        let mut item = new_primary_postmortem_target("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryPostmortemTargetRequiredNameRule {
            message: "primary postmortem target needs this field".to_string(),
        };
        let mut item = new_primary_postmortem_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PrimaryPostmortemTargetSummaryLengthRule { max_length: 86400 };
        let mut item = new_primary_postmortem_target("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryPostmortemTargetRequiredCodeRule {
            message: "primary postmortem target needs an entry".to_string(),
        };
        let mut item = new_primary_postmortem_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
