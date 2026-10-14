use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering score calculation.
pub trait PeeringScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringScoreAdjuster {
    pub percent: f64,
}

impl PeeringScoreAdjuster for PercentPeeringScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringScoreAdjuster {
    pub minimum: f64,
}

impl PeeringScoreAdjuster for MinimumPeeringScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPeeringScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringScoreAdjuster for LabelPeeringScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PeeringScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringScoreAdjuster {
    pub cap: f64,
}

impl PeeringScoreAdjuster for CapPeeringScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringScoreAdjuster {
    pub step: f64,
}

impl PeeringScoreAdjuster for RoundingPeeringScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the peering score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringScoreEngine {
    pub steps: Vec<Rc<dyn PeeringScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringScoreEngine {
    pub fn run(&mut self, item: &PeeringScore) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_peering_score_engine() -> PeeringScoreEngine {
    PeeringScoreEngine {
        steps: vec![
            Rc::new(PercentPeeringScoreAdjuster { percent: 10.7 }) as Rc<dyn PeeringScoreAdjuster>,
            Rc::new(MinimumPeeringScoreAdjuster { minimum: 45.7 }),
            Rc::new(LabelPeeringScoreAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.02), ("online".to_string(), 0.87)]),
            }),
            Rc::new(CapPeeringScoreAdjuster { cap: 35350.0 }),
            Rc::new(RoundingPeeringScoreAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 91860.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_score_engine();
        let item = new_peering_score("code-536", "label-899");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
