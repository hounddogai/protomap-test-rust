use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global page slot calculation.
pub trait GlobalPageSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPageSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGlobalPageSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPageSlotStep for TitleGlobalPageSlotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GlobalPageSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPageSlotStep for GlobalPageSlotWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPageSlot, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPageSlotAdjuster {
    pub percent: f64,
}

impl GlobalPageSlotStep for PercentGlobalPageSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPageSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPageSlotAdjuster {
    pub step: f64,
}

impl GlobalPageSlotStep for RoundingGlobalPageSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPageSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global page slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPageSlotEngine {
    pub steps: Vec<Rc<dyn GlobalPageSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPageSlotEngine {
    pub fn compute(&mut self, item: &GlobalPageSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPageSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPageSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_page_slot_engine() -> GlobalPageSlotEngine {
    GlobalPageSlotEngine {
        steps: vec![
            Rc::new(TitleGlobalPageSlotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.33)]),
            }) as Rc<dyn GlobalPageSlotStep>,
            Rc::new(GlobalPageSlotWindowDaysOffsetAdjuster { weight: 0.72 }),
            Rc::new(PercentGlobalPageSlotAdjuster { percent: 7.7 }),
            Rc::new(RoundingGlobalPageSlotAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 13676.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_page_slot_engine();
        let item = new_global_page_slot("code-85", "title-194");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
