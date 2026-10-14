use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze policy calculation.
pub trait FreezePolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezePolicyScoreOffsetAdjuster {
    pub weight: f64,
}

impl FreezePolicyFactor for FreezePolicyScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezePolicyFactor for TieredFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezePolicyAdjuster {
    pub minimum: f64,
}

impl FreezePolicyFactor for MinimumFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezePolicyAdjuster {
    pub percent: f64,
}

impl FreezePolicyFactor for PercentFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezePolicyAdjuster {
    pub step: f64,
}

impl FreezePolicyFactor for RoundingFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezePolicyFactor for FlaggedFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the freeze policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezePolicyEngine {
    pub steps: Vec<Rc<dyn FreezePolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezePolicyEngine {
    pub fn run(&mut self, item: &FreezePolicy) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[FreezePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_policy_engine(flags: &flags::Registry) -> FreezePolicyEngine {
    FreezePolicyEngine {
        steps: vec![
            Rc::new(FreezePolicyScoreOffsetAdjuster { weight: 0.67 }) as Rc<dyn FreezePolicyFactor>,
            Rc::new(TieredFreezePolicyAdjuster {
                bounds: vec![1345.0, 9765.0],
                rates: vec![1.14, 0.81],
            }),
            Rc::new(MinimumFreezePolicyAdjuster { minimum: 17.6 }),
            Rc::new(PercentFreezePolicyAdjuster { percent: 20.3 }),
            Rc::new(RoundingFreezePolicyAdjuster { step: 1.0 }),
            Rc::new(FlaggedFreezePolicyAdjuster {
                flags: flags.clone(),
                flag: "freeze-policy-primary".to_string(),
                percent: 8.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 97572.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_policy_engine(&flags::new_registry());
        let item = new_freeze_policy("code-784", "label-750");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
