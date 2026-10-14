use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier variance calculation.
pub trait VerifierVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierVarianceAdjuster {
    pub minimum: f64,
}

impl VerifierVarianceFactor for MinimumVerifierVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierVarianceAdjuster {
    pub step: f64,
}

impl VerifierVarianceFactor for RoundingVerifierVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierVarianceFactor for FlaggedVerifierVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierVarianceAdjuster {
    pub cap: f64,
}

impl VerifierVarianceFactor for CapVerifierVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the verifier variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierVarianceEngine {
    pub steps: Vec<Rc<dyn VerifierVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierVarianceEngine {
    pub fn run(&mut self, item: &VerifierVariance) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierVarianceFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[VerifierVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_variance_engine(flags: &flags::Registry) -> VerifierVarianceEngine {
    VerifierVarianceEngine {
        steps: vec![
            Rc::new(MinimumVerifierVarianceAdjuster { minimum: 19.5 }) as Rc<dyn VerifierVarianceFactor>,
            Rc::new(RoundingVerifierVarianceAdjuster { step: 0.05 }),
            Rc::new(FlaggedVerifierVarianceAdjuster {
                flags: flags.clone(),
                flag: "verifier-variance-weighted".to_string(),
                percent: -0.7,
            }),
            Rc::new(CapVerifierVarianceAdjuster { cap: 42643.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 20538.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_variance_engine(&flags::new_registry());
        let item = new_verifier_variance("code-610", "name-834");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
