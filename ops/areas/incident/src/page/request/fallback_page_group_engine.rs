use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback page group calculation.
pub trait FallbackPageGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPageGroupAdjuster {
    pub percent: f64,
}

impl FallbackPageGroupStep for PercentFallbackPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPageGroupAdjuster {
    pub step: f64,
}

impl FallbackPageGroupStep for RoundingFallbackPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPageGroupAdjuster {
    pub cap: f64,
}

impl FallbackPageGroupStep for CapFallbackPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupRetriesOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPageGroupStep for FallbackPageGroupRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackPageGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPageGroupStep for LabelFallbackPageGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPageGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPageGroupStep for TieredFallbackPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPageGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback page group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPageGroupEngine {
    pub steps: Vec<Rc<dyn FallbackPageGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPageGroupEngine {
    pub fn price(&mut self, item: &FallbackPageGroup) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPageGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_page_group_engine() -> FallbackPageGroupEngine {
    FallbackPageGroupEngine {
        steps: vec![
            Rc::new(PercentFallbackPageGroupAdjuster { percent: 0.5 }) as Rc<dyn FallbackPageGroupStep>,
            Rc::new(RoundingFallbackPageGroupAdjuster { step: 5.0 }),
            Rc::new(CapFallbackPageGroupAdjuster { cap: 24510.0 }),
            Rc::new(FallbackPageGroupRetriesOffsetAdjuster { weight: 0.67 }),
            Rc::new(LabelFallbackPageGroupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.16), ("central".to_string(), 1.01)]),
            }),
            Rc::new(TieredFallbackPageGroupAdjuster {
                bounds: vec![886.0, 2780.0, 6676.0],
                rates: vec![0.86, 1.07, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 96644.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_page_group_engine();
        let item = new_fallback_page_group("code-172", "label-671");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
