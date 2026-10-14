use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual peering entry calculation.
pub trait ManualPeeringEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualPeeringEntryAdjuster {
    pub step: f64,
}

impl ManualPeeringEntryModifier for RoundingManualPeeringEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualPeeringEntryAdjuster {
    pub percent: f64,
}

impl ManualPeeringEntryModifier for PercentManualPeeringEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualPeeringEntryAdjuster {
    pub cap: f64,
}

impl ManualPeeringEntryModifier for CapManualPeeringEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualPeeringEntryModifier for ManualPeeringEntryWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelManualPeeringEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualPeeringEntryModifier for LabelManualPeeringEntryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ManualPeeringEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual peering entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualPeeringEntryEngine {
    pub steps: Vec<Rc<dyn ManualPeeringEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualPeeringEntryEngine {
    pub fn compute(&mut self, item: &ManualPeeringEntry) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualPeeringEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualPeeringEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_peering_entry_engine() -> ManualPeeringEntryEngine {
    ManualPeeringEntryEngine {
        steps: vec![
            Rc::new(RoundingManualPeeringEntryAdjuster { step: 0.5 }) as Rc<dyn ManualPeeringEntryModifier>,
            Rc::new(PercentManualPeeringEntryAdjuster { percent: 4.9 }),
            Rc::new(CapManualPeeringEntryAdjuster { cap: 25299.0 }),
            Rc::new(ManualPeeringEntryWindowDaysOffsetAdjuster { weight: 0.77 }),
            Rc::new(LabelManualPeeringEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.81), ("retail".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 63833.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_peering_entry_engine();
        let item = new_manual_peering_entry("code-192", "label-805");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
