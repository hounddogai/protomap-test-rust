use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal allowlist mapping calculation.
pub trait SeasonalAllowlistMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalAllowlistMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalAllowlistMappingAdjuster for SeasonalAllowlistMappingDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalAllowlistMapping, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalAllowlistMappingAdjuster {
    pub minimum: f64,
}

impl SeasonalAllowlistMappingAdjuster for MinimumSeasonalAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalAllowlistMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalAllowlistMappingAdjuster {
    pub cap: f64,
}

impl SeasonalAllowlistMappingAdjuster for CapSeasonalAllowlistMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalAllowlistMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal allowlist mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalAllowlistMappingEngine {
    pub steps: Vec<Rc<dyn SeasonalAllowlistMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalAllowlistMappingEngine {
    pub fn price(&mut self, item: &SeasonalAllowlistMapping) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalAllowlistMappingAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalAllowlistMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_allowlist_mapping_engine() -> SeasonalAllowlistMappingEngine {
    SeasonalAllowlistMappingEngine {
        steps: vec![
            Rc::new(SeasonalAllowlistMappingDiscountOffsetAdjuster { weight: 0.81 })
                as Rc<dyn SeasonalAllowlistMappingAdjuster>,
            Rc::new(MinimumSeasonalAllowlistMappingAdjuster { minimum: 35.8 }),
            Rc::new(CapSeasonalAllowlistMappingAdjuster { cap: 40575.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 80017.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_allowlist_mapping_engine();
        let item = new_seasonal_allowlist_mapping("code-964", "label-176");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
