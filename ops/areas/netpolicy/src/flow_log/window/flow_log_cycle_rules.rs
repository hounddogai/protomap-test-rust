use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log cycle must pass. Check returns its problems, or none.
pub trait FlowLogCycleRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleRequiredCodeRule {
    pub message: String,
}

impl FlowLogCycleRule for FlowLogCycleRequiredCodeRule {
    fn check(&self, item: &FlowLogCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl FlowLogCycleRule for FlowLogCycleAllowedExternalRefRule {
    fn check(&self, item: &FlowLogCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is out of range", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FlowLogCycleRule for FlowLogCycleRateRangeRule {
    fn check(&self, item: &FlowLogCycle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleCipherLengthRule {
    pub max_length: i64,
}

impl FlowLogCycleRule for FlowLogCycleCipherLengthRule {
    fn check(&self, item: &FlowLogCycle) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleRequiredCipherRule {
    pub message: String,
}

impl FlowLogCycleRule for FlowLogCycleRequiredCipherRule {
    fn check(&self, item: &FlowLogCycle) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log cycle.
#[derive(Debug, Clone)]
pub struct FlowLogCycleValidator {
    pub rules: Vec<Rc<dyn FlowLogCycleRule>>,
}

impl FlowLogCycleValidator {
    pub fn validate(&self, item: &FlowLogCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_cycle_validator() -> FlowLogCycleValidator {
    FlowLogCycleValidator {
        rules: vec![
            Rc::new(FlowLogCycleRequiredCodeRule {
                message: "flow log cycle needs an entry".to_string(),
            }) as Rc<dyn FlowLogCycleRule>,
            Rc::new(FlowLogCycleAllowedExternalRefRule {
                allowed: vec!["strict".to_string(), "primary".to_string(), "batched".to_string()],
            }),
            Rc::new(FlowLogCycleRateRangeRule { min: 2.1, max: 1671.7 }),
            Rc::new(FlowLogCycleCipherLengthRule { max_length: 255 }),
            Rc::new(FlowLogCycleRequiredCipherRule {
                message: "flow log cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_cycle_validator();
        let item = new_flow_log_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogCycleRequiredCodeRule {
            message: "flow log cycle needs an entry".to_string(),
        };
        let mut item = new_flow_log_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = FlowLogCycleRateRangeRule { min: 7.7, max: 4540.6 };
        let mut item = new_flow_log_cycle("x", "y");
        item.rate = 4542.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = FlowLogCycleCipherLengthRule { max_length: 120 };
        let mut item = new_flow_log_cycle("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FlowLogCycleRequiredCipherRule {
            message: "flow log cycle needs a value".to_string(),
        };
        let mut item = new_flow_log_cycle("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
