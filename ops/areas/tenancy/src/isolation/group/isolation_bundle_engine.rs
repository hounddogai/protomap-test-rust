use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation bundle calculation.
pub trait IsolationBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationBundleAdjuster for FlaggedIsolationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationBundleAdjuster {
    pub minimum: f64,
}

impl IsolationBundleAdjuster for MinimumIsolationBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationBundleAdjuster {
    pub cap: f64,
}

impl IsolationBundleAdjuster for CapIsolationBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationBundleAdjuster for TieredIsolationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the isolation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationBundleEngine {
    pub steps: Vec<Rc<dyn IsolationBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationBundleEngine {
    pub fn compute(&mut self, item: &IsolationBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[IsolationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_bundle_engine(flags: &flags::Registry) -> IsolationBundleEngine {
    IsolationBundleEngine {
        steps: vec![
            Rc::new(FlaggedIsolationBundleAdjuster {
                flags: flags.clone(),
                flag: "isolation-bundle-strict".to_string(),
                percent: 4.3,
            }) as Rc<dyn IsolationBundleAdjuster>,
            Rc::new(MinimumIsolationBundleAdjuster { minimum: 47.5 }),
            Rc::new(CapIsolationBundleAdjuster { cap: 39221.0 }),
            Rc::new(TieredIsolationBundleAdjuster {
                bounds: vec![3789.0, 8896.0],
                rates: vec![0.93, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 39706.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_bundle_engine(&flags::new_registry());
        let item = new_isolation_bundle("code-433", "title-492");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
