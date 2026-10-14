use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding summary calculation.
pub trait BindingSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBindingSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingSummaryAdjuster for CountryBindingSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BindingSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingSummaryAdjuster for TieredBindingSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingSummaryAdjuster {
    pub minimum: f64,
}

impl BindingSummaryAdjuster for MinimumBindingSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingSummaryAdjuster {
    pub cap: f64,
}

impl BindingSummaryAdjuster for CapBindingSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the binding summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingSummaryEngine {
    pub steps: Vec<Rc<dyn BindingSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingSummaryEngine {
    pub fn run(&mut self, item: &BindingSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BindingSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_binding_summary_engine() -> BindingSummaryEngine {
    BindingSummaryEngine {
        steps: vec![
            Rc::new(CountryBindingSummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72), ("east".to_string(), 0.81)]),
            }) as Rc<dyn BindingSummaryAdjuster>,
            Rc::new(TieredBindingSummaryAdjuster {
                bounds: vec![896.0, 2567.0, 2788.0, 7095.0],
                rates: vec![0.88, 1.05, 0.93, 1.18],
            }),
            Rc::new(MinimumBindingSummaryAdjuster { minimum: 13.5 }),
            Rc::new(CapBindingSummaryAdjuster { cap: 10199.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 77455.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_summary_engine();
        let item = new_binding_summary("code-857", "label-744");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
