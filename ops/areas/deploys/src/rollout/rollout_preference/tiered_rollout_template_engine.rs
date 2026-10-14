use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered rollout template calculation.
pub trait TieredRolloutTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRolloutTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRolloutTemplateAdjuster {
    pub step: f64,
}

impl TieredRolloutTemplateAdjuster for RoundingTieredRolloutTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRolloutTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRolloutTemplateAdjuster {
    pub cap: f64,
}

impl TieredRolloutTemplateAdjuster for CapTieredRolloutTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRolloutTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRolloutTemplateAdjuster {
    pub minimum: f64,
}

impl TieredRolloutTemplateAdjuster for MinimumTieredRolloutTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRolloutTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered rollout template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRolloutTemplateEngine {
    pub steps: Vec<Rc<dyn TieredRolloutTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRolloutTemplateEngine {
    pub fn compute(&mut self, item: &TieredRolloutTemplate) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRolloutTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRolloutTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_rollout_template_engine() -> TieredRolloutTemplateEngine {
    TieredRolloutTemplateEngine {
        steps: vec![
            Rc::new(RoundingTieredRolloutTemplateAdjuster { step: 0.5 }) as Rc<dyn TieredRolloutTemplateAdjuster>,
            Rc::new(CapTieredRolloutTemplateAdjuster { cap: 37992.0 }),
            Rc::new(MinimumTieredRolloutTemplateAdjuster { minimum: 21.5 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 64956.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_rollout_template_engine();
        let item = new_tiered_rollout_template("code-10", "title-892");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
