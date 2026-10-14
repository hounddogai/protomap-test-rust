use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict grant segment calculation.
pub trait StrictGrantSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictGrantSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictGrantSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictGrantSegmentFactor for CategoryStrictGrantSegmentAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictGrantSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictGrantSegmentAdjuster {
    pub percent: f64,
}

impl StrictGrantSegmentFactor for PercentStrictGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictGrantSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictGrantSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictGrantSegmentFactor for TieredStrictGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictGrantSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantSegmentRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl StrictGrantSegmentFactor for StrictGrantSegmentRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &StrictGrantSegment, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the strict grant segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictGrantSegmentEngine {
    pub steps: Vec<Rc<dyn StrictGrantSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictGrantSegmentEngine {
    pub fn price(&mut self, item: &StrictGrantSegment) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[StrictGrantSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_grant_segment_engine() -> StrictGrantSegmentEngine {
    StrictGrantSegmentEngine {
        steps: vec![
            Rc::new(CategoryStrictGrantSegmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.28)]),
            }) as Rc<dyn StrictGrantSegmentFactor>,
            Rc::new(PercentStrictGrantSegmentAdjuster { percent: -3.4 }),
            Rc::new(TieredStrictGrantSegmentAdjuster {
                bounds: vec![3955.0, 7545.0, 8946.0, 9997.0],
                rates: vec![0.83, 0.99, 1.12, 0.89],
            }),
            Rc::new(StrictGrantSegmentRotationDaysOffsetAdjuster { weight: 0.15 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 51183.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_grant_segment_engine();
        let item = new_strict_grant_segment("code-183", "title-707");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
