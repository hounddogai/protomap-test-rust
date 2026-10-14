use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending timeline policy calculation.
pub trait PendingTimelinePolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTimelinePolicyAdjuster {
    pub cap: f64,
}

impl PendingTimelinePolicyModifier for CapPendingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingTimelinePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingTimelinePolicyModifier for TieredPendingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTimelinePolicyAdjuster {
    pub step: f64,
}

impl PendingTimelinePolicyModifier for RoundingPendingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingTimelinePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTimelinePolicyModifier for LabelPendingTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelinePolicyMarginOffsetAdjuster {
    pub weight: f64,
}

impl PendingTimelinePolicyModifier for PendingTimelinePolicyMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTimelinePolicy, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the pending timeline policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTimelinePolicyEngine {
    pub steps: Vec<Rc<dyn PendingTimelinePolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTimelinePolicyEngine {
    pub fn evaluate(&mut self, item: &PendingTimelinePolicy) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingTimelinePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_timeline_policy_engine() -> PendingTimelinePolicyEngine {
    PendingTimelinePolicyEngine {
        steps: vec![
            Rc::new(CapPendingTimelinePolicyAdjuster { cap: 12163.0 }) as Rc<dyn PendingTimelinePolicyModifier>,
            Rc::new(TieredPendingTimelinePolicyAdjuster {
                bounds: vec![6799.0, 7012.0],
                rates: vec![0.97, 1.19],
            }),
            Rc::new(RoundingPendingTimelinePolicyAdjuster { step: 0.25 }),
            Rc::new(LabelPendingTimelinePolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.93)]),
            }),
            Rc::new(PendingTimelinePolicyMarginOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 72858.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_timeline_policy_engine();
        let item = new_pending_timeline_policy("code-261", "label-483");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
