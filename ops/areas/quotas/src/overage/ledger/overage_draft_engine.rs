use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage draft calculation.
pub trait OverageDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageDraftAdjuster {
    pub cap: f64,
}

impl OverageDraftModifier for CapOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageDraftModifier for TieredOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageDraftAdjuster {
    pub percent: f64,
}

impl OverageDraftModifier for PercentOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageDraftMarginOffsetAdjuster {
    pub weight: f64,
}

impl OverageDraftModifier for OverageDraftMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &OverageDraft, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageDraftAdjuster {
    pub step: f64,
}

impl OverageDraftModifier for RoundingOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageDraftEngine {
    pub steps: Vec<Rc<dyn OverageDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageDraftEngine {
    pub fn run(&mut self, item: &OverageDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_overage_draft_engine() -> OverageDraftEngine {
    OverageDraftEngine {
        steps: vec![
            Rc::new(CapOverageDraftAdjuster { cap: 48967.0 }) as Rc<dyn OverageDraftModifier>,
            Rc::new(TieredOverageDraftAdjuster {
                bounds: vec![4530.0, 7851.0],
                rates: vec![0.84, 1.19],
            }),
            Rc::new(PercentOverageDraftAdjuster { percent: 7.0 }),
            Rc::new(OverageDraftMarginOffsetAdjuster { weight: 0.98 }),
            Rc::new(RoundingOverageDraftAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 53517.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_draft_engine();
        let item = new_overage_draft("code-927", "title-565");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
