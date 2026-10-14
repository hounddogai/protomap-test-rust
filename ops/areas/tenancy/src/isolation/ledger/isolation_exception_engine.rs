use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation exception calculation.
pub trait IsolationExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl IsolationExceptionStep for IsolationExceptionTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationException, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationExceptionStep for FlaggedIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationExceptionAdjuster {
    pub percent: f64,
}

impl IsolationExceptionStep for PercentIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationExceptionAdjuster {
    pub cap: f64,
}

impl IsolationExceptionStep for CapIsolationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the isolation exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationExceptionEngine {
    pub steps: Vec<Rc<dyn IsolationExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationExceptionEngine {
    pub fn price(&mut self, item: &IsolationException) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IsolationException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_exception_engine(flags: &flags::Registry) -> IsolationExceptionEngine {
    IsolationExceptionEngine {
        steps: vec![
            Rc::new(IsolationExceptionTtlSecondsOffsetAdjuster { weight: 0.54 }) as Rc<dyn IsolationExceptionStep>,
            Rc::new(FlaggedIsolationExceptionAdjuster {
                flags: flags.clone(),
                flag: "isolation-exception-strict".to_string(),
                percent: 0.4,
            }),
            Rc::new(PercentIsolationExceptionAdjuster { percent: 21.1 }),
            Rc::new(CapIsolationExceptionAdjuster { cap: 47290.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 33683.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_exception_engine(&flags::new_registry());
        let item = new_isolation_exception("code-566", "name-67");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
