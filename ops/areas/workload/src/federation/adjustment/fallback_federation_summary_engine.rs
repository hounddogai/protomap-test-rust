use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback federation summary calculation.
pub trait FallbackFederationSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackFederationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackFederationSummaryStep for LabelFallbackFederationSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackFederationSummaryAdjuster {
    pub percent: f64,
}

impl FallbackFederationSummaryStep for PercentFallbackFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryRateOffsetAdjuster {
    pub weight: f64,
}

impl FallbackFederationSummaryStep for FallbackFederationSummaryRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackFederationSummaryAdjuster {
    pub minimum: f64,
}

impl FallbackFederationSummaryStep for MinimumFallbackFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackFederationSummaryAdjuster {
    pub step: f64,
}

impl FallbackFederationSummaryStep for RoundingFallbackFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackFederationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback federation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackFederationSummaryEngine {
    pub steps: Vec<Rc<dyn FallbackFederationSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackFederationSummaryEngine {
    pub fn run(&mut self, item: &FallbackFederationSummary) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackFederationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_federation_summary_engine() -> FallbackFederationSummaryEngine {
    FallbackFederationSummaryEngine {
        steps: vec![
            Rc::new(LabelFallbackFederationSummaryAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.76)]),
            }) as Rc<dyn FallbackFederationSummaryStep>,
            Rc::new(PercentFallbackFederationSummaryAdjuster { percent: 0.8 }),
            Rc::new(FallbackFederationSummaryRateOffsetAdjuster { weight: 0.98 }),
            Rc::new(MinimumFallbackFederationSummaryAdjuster { minimum: 13.6 }),
            Rc::new(RoundingFallbackFederationSummaryAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 40067.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_federation_summary_engine();
        let item = new_fallback_federation_summary("code-123", "label-938");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
