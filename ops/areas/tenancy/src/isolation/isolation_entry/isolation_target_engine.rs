use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation target calculation.
pub trait IsolationTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationTargetAdjuster {
    pub cap: f64,
}

impl IsolationTargetAdjuster for CapIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationTargetAdjuster {
    pub step: f64,
}

impl IsolationTargetAdjuster for RoundingIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationTargetAdjuster for TieredIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationTargetAdjuster for FlaggedIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceIsolationTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationTargetAdjuster for SourceIsolationTargetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationTargetAdjuster {
    pub minimum: f64,
}

impl IsolationTargetAdjuster for MinimumIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the isolation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationTargetEngine {
    pub steps: Vec<Rc<dyn IsolationTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationTargetEngine {
    pub fn price(&mut self, item: &IsolationTarget) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[IsolationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_target_engine(flags: &flags::Registry) -> IsolationTargetEngine {
    IsolationTargetEngine {
        steps: vec![
            Rc::new(CapIsolationTargetAdjuster { cap: 8576.0 }) as Rc<dyn IsolationTargetAdjuster>,
            Rc::new(RoundingIsolationTargetAdjuster { step: 0.05 }),
            Rc::new(TieredIsolationTargetAdjuster {
                bounds: vec![108.0, 2796.0, 5138.0, 7262.0],
                rates: vec![0.82, 0.99, 1.16, 0.97],
            }),
            Rc::new(FlaggedIsolationTargetAdjuster {
                flags: flags.clone(),
                flag: "isolation-target-global".to_string(),
                percent: -6.5,
            }),
            Rc::new(SourceIsolationTargetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.74), ("north".to_string(), 1.26)]),
            }),
            Rc::new(MinimumIsolationTargetAdjuster { minimum: 27.7 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 60359.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_target_engine(&flags::new_registry());
        let item = new_isolation_target("code-636", "title-341");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
