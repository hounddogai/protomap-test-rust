use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending probe draft calculation.
pub trait PendingProbeDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingProbeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingProbeDraftAdjuster {
    pub minimum: f64,
}

impl PendingProbeDraftModifier for MinimumPendingProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingProbeDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingProbeDraftAdjuster {
    pub cap: f64,
}

impl PendingProbeDraftModifier for CapPendingProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingProbeDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraftBurstOffsetAdjuster {
    pub weight: f64,
}

impl PendingProbeDraftModifier for PendingProbeDraftBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PendingProbeDraft, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingProbeDraftAdjuster {
    pub percent: f64,
}

impl PendingProbeDraftModifier for PercentPendingProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingProbeDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending probe draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingProbeDraftEngine {
    pub steps: Vec<Rc<dyn PendingProbeDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingProbeDraftEngine {
    pub fn price(&mut self, item: &PendingProbeDraft) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingProbeDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingProbeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_probe_draft_engine() -> PendingProbeDraftEngine {
    PendingProbeDraftEngine {
        steps: vec![
            Rc::new(MinimumPendingProbeDraftAdjuster { minimum: 21.9 }) as Rc<dyn PendingProbeDraftModifier>,
            Rc::new(CapPendingProbeDraftAdjuster { cap: 1844.0 }),
            Rc::new(PendingProbeDraftBurstOffsetAdjuster { weight: 1.85 }),
            Rc::new(PercentPendingProbeDraftAdjuster { percent: 13.2 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 67118.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_probe_draft_engine();
        let item = new_pending_probe_draft("code-770", "label-729");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
