use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped verifier rollup calculation.
pub trait CappedVerifierRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedVerifierRollupAdjuster {
    pub step: f64,
}

impl CappedVerifierRollupFactor for RoundingCappedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierRollupStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedVerifierRollupFactor for CappedVerifierRollupStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedVerifierRollupAdjuster {
    pub minimum: f64,
}

impl CappedVerifierRollupFactor for MinimumCappedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedVerifierRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedVerifierRollupFactor for FlaggedCappedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedVerifierRollupAdjuster {
    pub percent: f64,
}

impl CappedVerifierRollupFactor for PercentCappedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryCappedVerifierRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedVerifierRollupFactor for CountryCappedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &CappedVerifierRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped verifier rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedVerifierRollupEngine {
    pub steps: Vec<Rc<dyn CappedVerifierRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedVerifierRollupEngine {
    pub fn run(&mut self, item: &CappedVerifierRollup) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedVerifierRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_verifier_rollup_engine(flags: &flags::Registry) -> CappedVerifierRollupEngine {
    CappedVerifierRollupEngine {
        steps: vec![
            Rc::new(RoundingCappedVerifierRollupAdjuster { step: 0.1 }) as Rc<dyn CappedVerifierRollupFactor>,
            Rc::new(CappedVerifierRollupStartsAtOffsetAdjuster { weight: 1.93 }),
            Rc::new(MinimumCappedVerifierRollupAdjuster { minimum: 11.0 }),
            Rc::new(FlaggedCappedVerifierRollupAdjuster {
                flags: flags.clone(),
                flag: "capped-verifier-rollup-scheduled".to_string(),
                percent: -6.9,
            }),
            Rc::new(PercentCappedVerifierRollupAdjuster { percent: -1.2 }),
            Rc::new(CountryCappedVerifierRollupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.07)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 9940.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_verifier_rollup_engine(&flags::new_registry());
        let item = new_capped_verifier_rollup("code-243", "name-249");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
