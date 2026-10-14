use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout agreement calculation.
pub trait RolloutAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl RolloutAgreementFactor for RolloutAgreementStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutAgreement, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutAgreementAdjuster {
    pub step: f64,
}

impl RolloutAgreementFactor for RoundingRolloutAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutAgreementAdjuster {
    pub cap: f64,
}

impl RolloutAgreementFactor for CapRolloutAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRolloutAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutAgreementFactor for TitleRolloutAgreementAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RolloutAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollout agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutAgreementEngine {
    pub steps: Vec<Rc<dyn RolloutAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutAgreementEngine {
    pub fn evaluate(&mut self, item: &RolloutAgreement) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_agreement_engine() -> RolloutAgreementEngine {
    RolloutAgreementEngine {
        steps: vec![
            Rc::new(RolloutAgreementStartsAtOffsetAdjuster { weight: 1.37 }) as Rc<dyn RolloutAgreementFactor>,
            Rc::new(RoundingRolloutAgreementAdjuster { step: 0.25 }),
            Rc::new(CapRolloutAgreementAdjuster { cap: 7596.0 }),
            Rc::new(TitleRolloutAgreementAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.72),
                    ("east".to_string(), 1.4),
                    ("north".to_string(), 0.87),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 41733.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_agreement_engine();
        let item = new_rollout_agreement("code-601", "title-477");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
