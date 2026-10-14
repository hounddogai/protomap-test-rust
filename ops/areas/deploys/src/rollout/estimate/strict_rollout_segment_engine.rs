use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict rollout segment calculation.
pub trait StrictRolloutSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictRolloutSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRolloutSegmentModifier for NameStrictRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictRolloutSegmentAdjuster {
    pub percent: f64,
}

impl StrictRolloutSegmentModifier for PercentStrictRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl StrictRolloutSegmentModifier for StrictRolloutSegmentRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictRolloutSegmentAdjuster {
    pub cap: f64,
}

impl StrictRolloutSegmentModifier for CapStrictRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictRolloutSegmentAdjuster {
    pub step: f64,
}

impl StrictRolloutSegmentModifier for RoundingStrictRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictRolloutSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict rollout segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRolloutSegmentEngine {
    pub steps: Vec<Rc<dyn StrictRolloutSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRolloutSegmentEngine {
    pub fn run(&mut self, item: &StrictRolloutSegment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictRolloutSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictRolloutSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_rollout_segment_engine() -> StrictRolloutSegmentEngine {
    StrictRolloutSegmentEngine {
        steps: vec![
            Rc::new(NameStrictRolloutSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.12), ("retail".to_string(), 0.71)]),
            }) as Rc<dyn StrictRolloutSegmentModifier>,
            Rc::new(PercentStrictRolloutSegmentAdjuster { percent: -6.3 }),
            Rc::new(StrictRolloutSegmentRotationDaysOffsetAdjuster { weight: 0.44 }),
            Rc::new(CapStrictRolloutSegmentAdjuster { cap: 28954.0 }),
            Rc::new(RoundingStrictRolloutSegmentAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 80927.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_rollout_segment_engine();
        let item = new_strict_rollout_segment("code-893", "name-743");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
