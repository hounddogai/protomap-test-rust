use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering exception calculation.
pub trait PeeringExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringExceptionAdjuster {
    pub step: f64,
}

impl PeeringExceptionAdjuster for RoundingPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringExceptionAdjuster {
    pub percent: f64,
}

impl PeeringExceptionAdjuster for PercentPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePeeringExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringExceptionAdjuster for SourcePeeringExceptionAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PeeringException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the peering exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringExceptionEngine {
    pub steps: Vec<Rc<dyn PeeringExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringExceptionEngine {
    pub fn run(&mut self, item: &PeeringException) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_peering_exception_engine() -> PeeringExceptionEngine {
    PeeringExceptionEngine {
        steps: vec![
            Rc::new(RoundingPeeringExceptionAdjuster { step: 0.25 }) as Rc<dyn PeeringExceptionAdjuster>,
            Rc::new(PercentPeeringExceptionAdjuster { percent: 24.5 }),
            Rc::new(SourcePeeringExceptionAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.35),
                    ("online".to_string(), 0.89),
                    ("north".to_string(), 1.38),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 18097.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_exception_engine();
        let item = new_peering_exception("code-871", "label-218");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
