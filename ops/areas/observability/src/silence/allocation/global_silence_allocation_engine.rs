use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global silence allocation calculation.
pub trait GlobalSilenceAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSilenceAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalSilenceAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalSilenceAllocationFactor for FlaggedGlobalSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalSilenceAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalSilenceAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalSilenceAllocationFactor for TieredGlobalSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalSilenceAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeGlobalSilenceAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalSilenceAllocationFactor for ScopeGlobalSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &GlobalSilenceAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalSilenceAllocationAdjuster {
    pub percent: f64,
}

impl GlobalSilenceAllocationFactor for PercentGlobalSilenceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalSilenceAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global silence allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSilenceAllocationEngine {
    pub steps: Vec<Rc<dyn GlobalSilenceAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSilenceAllocationEngine {
    pub fn price(&mut self, item: &GlobalSilenceAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalSilenceAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_silence_allocation_engine(flags: &flags::Registry) -> GlobalSilenceAllocationEngine {
    GlobalSilenceAllocationEngine {
        steps: vec![
            Rc::new(FlaggedGlobalSilenceAllocationAdjuster {
                flags: flags.clone(),
                flag: "global-silence-allocation-adaptive".to_string(),
                percent: 6.6,
            }) as Rc<dyn GlobalSilenceAllocationFactor>,
            Rc::new(TieredGlobalSilenceAllocationAdjuster {
                bounds: vec![92.0, 4768.0, 7060.0, 9338.0],
                rates: vec![1.2, 1.15, 0.95, 1.1],
            }),
            Rc::new(ScopeGlobalSilenceAllocationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.73)]),
            }),
            Rc::new(PercentGlobalSilenceAllocationAdjuster { percent: 15.4 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 76775.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_silence_allocation_engine(&flags::new_registry());
        let item = new_global_silence_allocation("code-933", "label-195");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
