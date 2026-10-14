use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query profile calculation.
pub trait QueryProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryProfileAdjuster {
    pub cap: f64,
}

impl QueryProfileFactor for CapQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl QueryProfileFactor for QueryProfileExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &QueryProfile, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryProfileFactor for TieredQueryProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the query profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryProfileEngine {
    pub steps: Vec<Rc<dyn QueryProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryProfileEngine {
    pub fn compute(&mut self, item: &QueryProfile) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[QueryProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_profile_engine() -> QueryProfileEngine {
    QueryProfileEngine {
        steps: vec![
            Rc::new(CapQueryProfileAdjuster { cap: 16790.0 }) as Rc<dyn QueryProfileFactor>,
            Rc::new(QueryProfileExpiresAtOffsetAdjuster { weight: 0.09 }),
            Rc::new(TieredQueryProfileAdjuster {
                bounds: vec![2427.0, 2843.0, 3133.0, 6658.0],
                rates: vec![0.84, 1.12, 1.18, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 68117.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_profile_engine();
        let item = new_query_profile("code-945", "name-952");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
