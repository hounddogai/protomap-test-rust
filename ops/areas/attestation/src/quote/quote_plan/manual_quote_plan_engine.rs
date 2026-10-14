use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual quote plan calculation.
pub trait ManualQuotePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualQuotePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitManualQuotePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualQuotePlanFactor for UnitManualQuotePlanAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ManualQuotePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualQuotePlanAdjuster {
    pub percent: f64,
}

impl ManualQuotePlanFactor for PercentManualQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualQuotePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualQuotePlanAdjuster {
    pub cap: f64,
}

impl ManualQuotePlanFactor for CapManualQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualQuotePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ManualQuotePlanFactor for ManualQuotePlanFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ManualQuotePlan, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the manual quote plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualQuotePlanEngine {
    pub steps: Vec<Rc<dyn ManualQuotePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualQuotePlanEngine {
    pub fn price(&mut self, item: &ManualQuotePlan) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualQuotePlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualQuotePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_quote_plan_engine() -> ManualQuotePlanEngine {
    ManualQuotePlanEngine {
        steps: vec![
            Rc::new(UnitManualQuotePlanAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.33), ("central".to_string(), 1.26)]),
            }) as Rc<dyn ManualQuotePlanFactor>,
            Rc::new(PercentManualQuotePlanAdjuster { percent: 6.8 }),
            Rc::new(CapManualQuotePlanAdjuster { cap: 40573.0 }),
            Rc::new(ManualQuotePlanFloorValueOffsetAdjuster { weight: 0.72 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 92560.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_quote_plan_engine();
        let item = new_manual_quote_plan("code-745", "title-460");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
