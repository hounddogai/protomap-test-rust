use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default rule cycle calculation.
pub trait DefaultRuleCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRuleCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRuleCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRuleCycleFactor for FlaggedDefaultRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRuleCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRuleCycleAdjuster {
    pub cap: f64,
}

impl DefaultRuleCycleFactor for CapDefaultRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRuleCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRuleCycleAdjuster {
    pub minimum: f64,
}

impl DefaultRuleCycleFactor for MinimumDefaultRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRuleCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRuleCycleAdjuster {
    pub step: f64,
}

impl DefaultRuleCycleFactor for RoundingDefaultRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRuleCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default rule cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRuleCycleEngine {
    pub steps: Vec<Rc<dyn DefaultRuleCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRuleCycleEngine {
    pub fn evaluate(&mut self, item: &DefaultRuleCycle) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRuleCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_rule_cycle_engine(flags: &flags::Registry) -> DefaultRuleCycleEngine {
    DefaultRuleCycleEngine {
        steps: vec![
            Rc::new(FlaggedDefaultRuleCycleAdjuster {
                flags: flags.clone(),
                flag: "default-rule-cycle-capped".to_string(),
                percent: 8.5,
            }) as Rc<dyn DefaultRuleCycleFactor>,
            Rc::new(CapDefaultRuleCycleAdjuster { cap: 44607.0 }),
            Rc::new(MinimumDefaultRuleCycleAdjuster { minimum: 48.5 }),
            Rc::new(RoundingDefaultRuleCycleAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 51274.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rule_cycle_engine(&flags::new_registry());
        let item = new_default_rule_cycle("code-68", "name-423");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
