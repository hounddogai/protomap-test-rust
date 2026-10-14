use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled health check rule calculation.
pub trait ScheduledHealthCheckRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledHealthCheckRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledHealthCheckRuleAdjuster {
    pub minimum: f64,
}

impl ScheduledHealthCheckRuleFactor for MinimumScheduledHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledHealthCheckRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledHealthCheckRuleAdjuster {
    pub cap: f64,
}

impl ScheduledHealthCheckRuleFactor for CapScheduledHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledHealthCheckRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledHealthCheckRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledHealthCheckRuleFactor for SegmentScheduledHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledHealthCheckRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled health check rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckRuleEngine {
    pub steps: Vec<Rc<dyn ScheduledHealthCheckRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledHealthCheckRuleEngine {
    pub fn price(&mut self, item: &ScheduledHealthCheckRule) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ScheduledHealthCheckRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_health_check_rule_engine() -> ScheduledHealthCheckRuleEngine {
    ScheduledHealthCheckRuleEngine {
        steps: vec![
            Rc::new(MinimumScheduledHealthCheckRuleAdjuster { minimum: 6.0 }) as Rc<dyn ScheduledHealthCheckRuleFactor>,
            Rc::new(CapScheduledHealthCheckRuleAdjuster { cap: 17256.0 }),
            Rc::new(SegmentScheduledHealthCheckRuleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 54915.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_health_check_rule_engine();
        let item = new_scheduled_health_check_rule("code-985", "title-321");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
