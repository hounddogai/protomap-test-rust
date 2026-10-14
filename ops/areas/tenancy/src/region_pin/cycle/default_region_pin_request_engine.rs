use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default region pin request calculation.
pub trait DefaultRegionPinRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultRegionPinRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRegionPinRequestStep for NameDefaultRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRegionPinRequestRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRegionPinRequestStep for DefaultRegionPinRequestRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultRegionPinRequestAdjuster {
    pub percent: f64,
}

impl DefaultRegionPinRequestStep for PercentDefaultRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRegionPinRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRegionPinRequestStep for TieredDefaultRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRegionPinRequestAdjuster {
    pub minimum: f64,
}

impl DefaultRegionPinRequestStep for MinimumDefaultRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRegionPinRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default region pin request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRegionPinRequestEngine {
    pub steps: Vec<Rc<dyn DefaultRegionPinRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRegionPinRequestEngine {
    pub fn compute(&mut self, item: &DefaultRegionPinRequest) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRegionPinRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_region_pin_request_engine() -> DefaultRegionPinRequestEngine {
    DefaultRegionPinRequestEngine {
        steps: vec![
            Rc::new(NameDefaultRegionPinRequestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.29)]),
            }) as Rc<dyn DefaultRegionPinRequestStep>,
            Rc::new(DefaultRegionPinRequestRateOffsetAdjuster { weight: 1.82 }),
            Rc::new(PercentDefaultRegionPinRequestAdjuster { percent: -15.2 }),
            Rc::new(TieredDefaultRegionPinRequestAdjuster {
                bounds: vec![743.0, 2606.0, 5182.0],
                rates: vec![0.84, 1.11, 1.0],
            }),
            Rc::new(MinimumDefaultRegionPinRequestAdjuster { minimum: 44.8 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 16558.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_region_pin_request_engine();
        let item = new_default_region_pin_request("code-993", "name-548");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
