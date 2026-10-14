use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier rollup calculation.
pub trait VerifierRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollupScoreOffsetAdjuster {
    pub weight: f64,
}

impl VerifierRollupFactor for VerifierRollupScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierRollupAdjuster {
    pub step: f64,
}

impl VerifierRollupFactor for RoundingVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierRollupAdjuster {
    pub cap: f64,
}

impl VerifierRollupFactor for CapVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameVerifierRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierRollupFactor for NameVerifierRollupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierRollupFactor for FlaggedVerifierRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the verifier rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierRollupEngine {
    pub steps: Vec<Rc<dyn VerifierRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierRollupEngine {
    pub fn compute(&mut self, item: &VerifierRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[VerifierRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_rollup_engine(flags: &flags::Registry) -> VerifierRollupEngine {
    VerifierRollupEngine {
        steps: vec![
            Rc::new(VerifierRollupScoreOffsetAdjuster { weight: 0.9 }) as Rc<dyn VerifierRollupFactor>,
            Rc::new(RoundingVerifierRollupAdjuster { step: 5.0 }),
            Rc::new(CapVerifierRollupAdjuster { cap: 41152.0 }),
            Rc::new(NameVerifierRollupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.25), ("online".to_string(), 0.77)]),
            }),
            Rc::new(FlaggedVerifierRollupAdjuster {
                flags: flags.clone(),
                flag: "verifier-rollup-batched".to_string(),
                percent: -3.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 17975.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_rollup_engine(&flags::new_registry());
        let item = new_verifier_rollup("code-694", "name-353");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
