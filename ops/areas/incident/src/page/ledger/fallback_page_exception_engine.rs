use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback page exception calculation.
pub trait FallbackPageExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPageException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPageExceptionAdjuster {
    pub cap: f64,
}

impl FallbackPageExceptionFactor for CapFallbackPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPageException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFallbackPageExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPageExceptionFactor for CategoryFallbackPageExceptionAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FallbackPageException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPageExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPageExceptionFactor for TieredFallbackPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPageException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback page exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPageExceptionEngine {
    pub steps: Vec<Rc<dyn FallbackPageExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPageExceptionEngine {
    pub fn compute(&mut self, item: &FallbackPageException) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPageException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_page_exception_engine() -> FallbackPageExceptionEngine {
    FallbackPageExceptionEngine {
        steps: vec![
            Rc::new(CapFallbackPageExceptionAdjuster { cap: 40210.0 }) as Rc<dyn FallbackPageExceptionFactor>,
            Rc::new(CategoryFallbackPageExceptionAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86), ("west".to_string(), 0.92)]),
            }),
            Rc::new(TieredFallbackPageExceptionAdjuster {
                bounds: vec![1587.0, 7693.0],
                rates: vec![1.18, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 20003.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_page_exception_engine();
        let item = new_fallback_page_exception("code-133", "label-327");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
