use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional region pin request calculation.
pub trait RegionalRegionPinRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRegionPinRequestRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRegionPinRequestFactor for RegionalRegionPinRequestRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRegionPinRequestAdjuster {
    pub cap: f64,
}

impl RegionalRegionPinRequestFactor for CapRegionalRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRegionPinRequestAdjuster {
    pub percent: f64,
}

impl RegionalRegionPinRequestFactor for PercentRegionalRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRegionPinRequestAdjuster {
    pub minimum: f64,
}

impl RegionalRegionPinRequestFactor for MinimumRegionalRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalRegionPinRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalRegionPinRequestFactor for TitleRegionalRegionPinRequestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalRegionPinRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional region pin request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRegionPinRequestEngine {
    pub steps: Vec<Rc<dyn RegionalRegionPinRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRegionPinRequestEngine {
    pub fn evaluate(&mut self, item: &RegionalRegionPinRequest) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRegionPinRequestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRegionPinRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_region_pin_request_engine() -> RegionalRegionPinRequestEngine {
    RegionalRegionPinRequestEngine {
        steps: vec![
            Rc::new(RegionalRegionPinRequestRevisionOffsetAdjuster { weight: 1.03 })
                as Rc<dyn RegionalRegionPinRequestFactor>,
            Rc::new(CapRegionalRegionPinRequestAdjuster { cap: 27762.0 }),
            Rc::new(PercentRegionalRegionPinRequestAdjuster { percent: -19.6 }),
            Rc::new(MinimumRegionalRegionPinRequestAdjuster { minimum: 27.9 }),
            Rc::new(TitleRegionalRegionPinRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.37), ("online".to_string(), 1.39)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 20367.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_region_pin_request_engine();
        let item = new_regional_region_pin_request("code-986", "title-576");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
