use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict ttl policy cycle calculation.
pub trait StrictTtlPolicyCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTtlPolicyCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictTtlPolicyCycleAdjuster {
    pub cap: f64,
}

impl StrictTtlPolicyCycleAdjuster for CapStrictTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictTtlPolicyCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictTtlPolicyCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictTtlPolicyCycleAdjuster for FlaggedStrictTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictTtlPolicyCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl StrictTtlPolicyCycleAdjuster for StrictTtlPolicyCycleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &StrictTtlPolicyCycle, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictTtlPolicyCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictTtlPolicyCycleAdjuster for TieredStrictTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictTtlPolicyCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict ttl policy cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTtlPolicyCycleEngine {
    pub steps: Vec<Rc<dyn StrictTtlPolicyCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTtlPolicyCycleEngine {
    pub fn compute(&mut self, item: &StrictTtlPolicyCycle) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[StrictTtlPolicyCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_ttl_policy_cycle_engine(flags: &flags::Registry) -> StrictTtlPolicyCycleEngine {
    StrictTtlPolicyCycleEngine {
        steps: vec![
            Rc::new(CapStrictTtlPolicyCycleAdjuster { cap: 4345.0 }) as Rc<dyn StrictTtlPolicyCycleAdjuster>,
            Rc::new(FlaggedStrictTtlPolicyCycleAdjuster {
                flags: flags.clone(),
                flag: "strict-ttl-policy-cycle-rolling".to_string(),
                percent: 5.9,
            }),
            Rc::new(StrictTtlPolicyCycleRevisionOffsetAdjuster { weight: 0.22 }),
            Rc::new(TieredStrictTtlPolicyCycleAdjuster {
                bounds: vec![2051.0, 3735.0],
                rates: vec![1.02, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 63701.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_ttl_policy_cycle_engine(&flags::new_registry());
        let item = new_strict_ttl_policy_cycle("code-441", "name-391");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
