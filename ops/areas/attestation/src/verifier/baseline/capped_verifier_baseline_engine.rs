use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped verifier baseline calculation.
pub trait CappedVerifierBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedVerifierBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedVerifierBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedVerifierBaselineStep for TitleCappedVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedVerifierBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedVerifierBaselineAdjuster {
    pub minimum: f64,
}

impl CappedVerifierBaselineStep for MinimumCappedVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedVerifierBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedVerifierBaselineAdjuster {
    pub percent: f64,
}

impl CappedVerifierBaselineStep for PercentCappedVerifierBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedVerifierBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped verifier baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedVerifierBaselineEngine {
    pub steps: Vec<Rc<dyn CappedVerifierBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedVerifierBaselineEngine {
    pub fn price(&mut self, item: &CappedVerifierBaseline) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedVerifierBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedVerifierBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_verifier_baseline_engine() -> CappedVerifierBaselineEngine {
    CappedVerifierBaselineEngine {
        steps: vec![
            Rc::new(TitleCappedVerifierBaselineAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.93),
                    ("west".to_string(), 0.86),
                    ("north".to_string(), 1.07),
                ]),
            }) as Rc<dyn CappedVerifierBaselineStep>,
            Rc::new(MinimumCappedVerifierBaselineAdjuster { minimum: 38.5 }),
            Rc::new(PercentCappedVerifierBaselineAdjuster { percent: 8.2 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 58768.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_verifier_baseline_engine();
        let item = new_capped_verifier_baseline("code-371", "title-369");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
