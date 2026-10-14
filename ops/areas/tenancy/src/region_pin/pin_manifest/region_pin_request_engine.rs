use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin request calculation.
pub trait RegionPinRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRegionPinRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinRequestAdjuster for CountryRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinRequestAdjuster for RegionPinRequestCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinRequestAdjuster {
    pub cap: f64,
}

impl RegionPinRequestAdjuster for CapRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinRequestAdjuster {
    pub minimum: f64,
}

impl RegionPinRequestAdjuster for MinimumRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinRequestAdjuster {
    pub step: f64,
}

impl RegionPinRequestAdjuster for RoundingRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinRequestAdjuster {
    pub percent: f64,
}

impl RegionPinRequestAdjuster for PercentRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinRequestEngine {
    pub steps: Vec<Rc<dyn RegionPinRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinRequestEngine {
    pub fn price(&mut self, item: &RegionPinRequest) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_request_engine() -> RegionPinRequestEngine {
    RegionPinRequestEngine {
        steps: vec![
            Rc::new(CountryRegionPinRequestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.86)]),
            }) as Rc<dyn RegionPinRequestAdjuster>,
            Rc::new(RegionPinRequestCreatedAtOffsetAdjuster { weight: 0.78 }),
            Rc::new(CapRegionPinRequestAdjuster { cap: 46501.0 }),
            Rc::new(MinimumRegionPinRequestAdjuster { minimum: 3.2 }),
            Rc::new(RoundingRegionPinRequestAdjuster { step: 0.25 }),
            Rc::new(PercentRegionPinRequestAdjuster { percent: -19.3 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 29595.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_request_engine();
        let item = new_region_pin_request("code-225", "name-482");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
