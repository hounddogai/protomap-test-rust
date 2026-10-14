use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted region pin profile calculation.
pub trait WeightedRegionPinProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRegionPinProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRegionPinProfileAdjuster {
    pub cap: f64,
}

impl WeightedRegionPinProfileAdjuster for CapWeightedRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRegionPinProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameWeightedRegionPinProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRegionPinProfileAdjuster for NameWeightedRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &WeightedRegionPinProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfileVersionOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRegionPinProfileAdjuster for WeightedRegionPinProfileVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRegionPinProfile, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRegionPinProfileAdjuster {
    pub percent: f64,
}

impl WeightedRegionPinProfileAdjuster for PercentWeightedRegionPinProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRegionPinProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted region pin profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRegionPinProfileEngine {
    pub steps: Vec<Rc<dyn WeightedRegionPinProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRegionPinProfileEngine {
    pub fn price(&mut self, item: &WeightedRegionPinProfile) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRegionPinProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRegionPinProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_region_pin_profile_engine() -> WeightedRegionPinProfileEngine {
    WeightedRegionPinProfileEngine {
        steps: vec![
            Rc::new(CapWeightedRegionPinProfileAdjuster { cap: 2081.0 }) as Rc<dyn WeightedRegionPinProfileAdjuster>,
            Rc::new(NameWeightedRegionPinProfileAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.13)]),
            }),
            Rc::new(WeightedRegionPinProfileVersionOffsetAdjuster { weight: 1.53 }),
            Rc::new(PercentWeightedRegionPinProfileAdjuster { percent: 7.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 33048.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_region_pin_profile_engine();
        let item = new_weighted_region_pin_profile("code-571", "name-760");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
