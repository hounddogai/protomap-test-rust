use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient isolation rollup calculation.
pub trait LenientIsolationRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupCapacityOffsetAdjuster {
    pub weight: f64,
}

impl LenientIsolationRollupAdjuster for LenientIsolationRollupCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientIsolationRollupAdjuster {
    pub minimum: f64,
}

impl LenientIsolationRollupAdjuster for MinimumLenientIsolationRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientIsolationRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientIsolationRollupAdjuster for TieredLenientIsolationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLenientIsolationRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientIsolationRollupAdjuster for RegionLenientIsolationRollupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientIsolationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientIsolationRollupAdjuster for FlaggedLenientIsolationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientIsolationRollupAdjuster {
    pub percent: f64,
}

impl LenientIsolationRollupAdjuster for PercentLenientIsolationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientIsolationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient isolation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientIsolationRollupEngine {
    pub steps: Vec<Rc<dyn LenientIsolationRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientIsolationRollupEngine {
    pub fn evaluate(&mut self, item: &LenientIsolationRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientIsolationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_isolation_rollup_engine(flags: &flags::Registry) -> LenientIsolationRollupEngine {
    LenientIsolationRollupEngine {
        steps: vec![
            Rc::new(LenientIsolationRollupCapacityOffsetAdjuster { weight: 0.67 })
                as Rc<dyn LenientIsolationRollupAdjuster>,
            Rc::new(MinimumLenientIsolationRollupAdjuster { minimum: 10.9 }),
            Rc::new(TieredLenientIsolationRollupAdjuster {
                bounds: vec![500.0, 2277.0, 6988.0, 7934.0],
                rates: vec![1.16, 0.88, 1.03, 0.85],
            }),
            Rc::new(RegionLenientIsolationRollupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35)]),
            }),
            Rc::new(FlaggedLenientIsolationRollupAdjuster {
                flags: flags.clone(),
                flag: "lenient-isolation-rollup-capped".to_string(),
                percent: 1.1,
            }),
            Rc::new(PercentLenientIsolationRollupAdjuster { percent: -7.6 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 36669.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_isolation_rollup_engine(&flags::new_registry());
        let item = new_lenient_isolation_rollup("code-58", "name-955");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
