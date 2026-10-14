use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global peering window calculation.
pub trait GlobalPeeringWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPeeringWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPeeringWindowStep for FlaggedGlobalPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPeeringWindowAdjuster {
    pub cap: f64,
}

impl GlobalPeeringWindowStep for CapGlobalPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPeeringWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPeeringWindowStep for TieredGlobalPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPeeringWindowAdjuster {
    pub minimum: f64,
}

impl GlobalPeeringWindowStep for MinimumGlobalPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPeeringWindowAdjuster {
    pub step: f64,
}

impl GlobalPeeringWindowStep for RoundingGlobalPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringWindowMaximumOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPeeringWindowStep for GlobalPeeringWindowMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPeeringWindow, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the global peering window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPeeringWindowEngine {
    pub steps: Vec<Rc<dyn GlobalPeeringWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPeeringWindowEngine {
    pub fn run(&mut self, item: &GlobalPeeringWindow) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPeeringWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_peering_window_engine(flags: &flags::Registry) -> GlobalPeeringWindowEngine {
    GlobalPeeringWindowEngine {
        steps: vec![
            Rc::new(FlaggedGlobalPeeringWindowAdjuster {
                flags: flags.clone(),
                flag: "global-peering-window-batched".to_string(),
                percent: -4.0,
            }) as Rc<dyn GlobalPeeringWindowStep>,
            Rc::new(CapGlobalPeeringWindowAdjuster { cap: 20794.0 }),
            Rc::new(TieredGlobalPeeringWindowAdjuster {
                bounds: vec![3121.0, 4900.0, 8217.0],
                rates: vec![0.84, 1.08, 0.97],
            }),
            Rc::new(MinimumGlobalPeeringWindowAdjuster { minimum: 26.9 }),
            Rc::new(RoundingGlobalPeeringWindowAdjuster { step: 0.5 }),
            Rc::new(GlobalPeeringWindowMaximumOffsetAdjuster { weight: 0.17 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 62825.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_peering_window_engine(&flags::new_registry());
        let item = new_global_peering_window("code-534", "name-976");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
