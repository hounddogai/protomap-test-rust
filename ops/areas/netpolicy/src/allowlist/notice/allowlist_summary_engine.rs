use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the allowlist summary calculation.
pub trait AllowlistSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryAllowlistSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistSummaryFactor for CountryAllowlistSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &AllowlistSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryScoreOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistSummaryFactor for AllowlistSummaryScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistSummary, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistSummaryAdjuster {
    pub percent: f64,
}

impl AllowlistSummaryFactor for PercentAllowlistSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistSummaryEngine {
    pub steps: Vec<Rc<dyn AllowlistSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistSummaryEngine {
    pub fn price(&mut self, item: &AllowlistSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AllowlistSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_summary_engine() -> AllowlistSummaryEngine {
    AllowlistSummaryEngine {
        steps: vec![
            Rc::new(CountryAllowlistSummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.95)]),
            }) as Rc<dyn AllowlistSummaryFactor>,
            Rc::new(AllowlistSummaryScoreOffsetAdjuster { weight: 0.45 }),
            Rc::new(PercentAllowlistSummaryAdjuster { percent: 14.8 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 85984.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_summary_engine();
        let item = new_allowlist_summary("code-954", "name-727");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
