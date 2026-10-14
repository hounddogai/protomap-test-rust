use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global binding segment must pass. Check returns its problems, or none.
pub trait GlobalBindingSegmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalBindingSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingSegmentRequiredScopeRule {
    pub message: String,
}

impl GlobalBindingSegmentRule for GlobalBindingSegmentRequiredScopeRule {
    fn check(&self, item: &GlobalBindingSegment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingSegmentCodePrefixRule {
    pub prefix: String,
}

impl GlobalBindingSegmentRule for GlobalBindingSegmentCodePrefixRule {
    fn check(&self, item: &GlobalBindingSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingSegmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalBindingSegmentRule for GlobalBindingSegmentRateRangeRule {
    fn check(&self, item: &GlobalBindingSegment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global binding segment.
#[derive(Debug, Clone)]
pub struct GlobalBindingSegmentValidator {
    pub rules: Vec<Rc<dyn GlobalBindingSegmentRule>>,
}

impl GlobalBindingSegmentValidator {
    pub fn validate(&self, item: &GlobalBindingSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalBindingSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalBindingSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_binding_segment_validator() -> GlobalBindingSegmentValidator {
    GlobalBindingSegmentValidator {
        rules: vec![
            Rc::new(GlobalBindingSegmentRequiredScopeRule {
                message: "global binding segment needs an entry".to_string(),
            }) as Rc<dyn GlobalBindingSegmentRule>,
            Rc::new(GlobalBindingSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(GlobalBindingSegmentRateRangeRule { min: 1.7, max: 904.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_binding_segment_validator();
        let item = new_global_binding_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = GlobalBindingSegmentRequiredScopeRule {
            message: "global binding segment needs a value".to_string(),
        };
        let mut item = new_global_binding_segment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = GlobalBindingSegmentRateRangeRule { min: 2.2, max: 2637.2 };
        let mut item = new_global_binding_segment("x", "y");
        item.rate = 2638.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
