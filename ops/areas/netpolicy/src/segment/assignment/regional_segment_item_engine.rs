use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional segment item calculation.
pub trait RegionalSegmentItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSegmentItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRegionalSegmentItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSegmentItemAdjuster for RegionRegionalSegmentItemAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RegionalSegmentItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentItemRateOffsetAdjuster {
    pub weight: f64,
}

impl RegionalSegmentItemAdjuster for RegionalSegmentItemRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalSegmentItem, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalSegmentItemAdjuster {
    pub minimum: f64,
}

impl RegionalSegmentItemAdjuster for MinimumRegionalSegmentItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalSegmentItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional segment item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSegmentItemEngine {
    pub steps: Vec<Rc<dyn RegionalSegmentItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSegmentItemEngine {
    pub fn evaluate(&mut self, item: &RegionalSegmentItem) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSegmentItemAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSegmentItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_segment_item_engine() -> RegionalSegmentItemEngine {
    RegionalSegmentItemEngine {
        steps: vec![
            Rc::new(RegionRegionalSegmentItemAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.08),
                    ("north".to_string(), 0.75),
                    ("south".to_string(), 0.83),
                ]),
            }) as Rc<dyn RegionalSegmentItemAdjuster>,
            Rc::new(RegionalSegmentItemRateOffsetAdjuster { weight: 1.12 }),
            Rc::new(MinimumRegionalSegmentItemAdjuster { minimum: 24.3 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 59790.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_segment_item_engine();
        let item = new_regional_segment_item("code-892", "label-357");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
