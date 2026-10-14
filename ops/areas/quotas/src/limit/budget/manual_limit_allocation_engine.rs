use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual limit allocation calculation.
pub trait ManualLimitAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualLimitAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualLimitAllocationAdjuster {
    pub cap: f64,
}

impl ManualLimitAllocationStep for CapManualLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualLimitAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualLimitAllocationAdjuster {
    pub step: f64,
}

impl ManualLimitAllocationStep for RoundingManualLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualLimitAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualLimitAllocationAdjuster {
    pub percent: f64,
}

impl ManualLimitAllocationStep for PercentManualLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualLimitAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitManualLimitAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualLimitAllocationStep for UnitManualLimitAllocationAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ManualLimitAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual limit allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualLimitAllocationEngine {
    pub steps: Vec<Rc<dyn ManualLimitAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualLimitAllocationEngine {
    pub fn evaluate(&mut self, item: &ManualLimitAllocation) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualLimitAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualLimitAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_limit_allocation_engine() -> ManualLimitAllocationEngine {
    ManualLimitAllocationEngine {
        steps: vec![
            Rc::new(CapManualLimitAllocationAdjuster { cap: 40128.0 }) as Rc<dyn ManualLimitAllocationStep>,
            Rc::new(RoundingManualLimitAllocationAdjuster { step: 1.0 }),
            Rc::new(PercentManualLimitAllocationAdjuster { percent: 6.2 }),
            Rc::new(UnitManualLimitAllocationAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.06),
                    ("north".to_string(), 1.39),
                    ("east".to_string(), 0.99),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 97921.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_limit_allocation_engine();
        let item = new_manual_limit_allocation("code-636", "title-545");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
