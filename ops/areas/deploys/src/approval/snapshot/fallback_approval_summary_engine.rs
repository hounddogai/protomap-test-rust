use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback approval summary calculation.
pub trait FallbackApprovalSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackApprovalSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackApprovalSummaryModifier for TieredFallbackApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackApprovalSummaryAdjuster {
    pub minimum: f64,
}

impl FallbackApprovalSummaryModifier for MinimumFallbackApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFallbackApprovalSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackApprovalSummaryModifier for CountryFallbackApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackApprovalSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackApprovalSummaryModifier for FlaggedFallbackApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackApprovalSummaryAdjuster {
    pub percent: f64,
}

impl FallbackApprovalSummaryModifier for PercentFallbackApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackApprovalSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback approval summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackApprovalSummaryEngine {
    pub steps: Vec<Rc<dyn FallbackApprovalSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackApprovalSummaryEngine {
    pub fn price(&mut self, item: &FallbackApprovalSummary) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackApprovalSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_approval_summary_engine(flags: &flags::Registry) -> FallbackApprovalSummaryEngine {
    FallbackApprovalSummaryEngine {
        steps: vec![
            Rc::new(TieredFallbackApprovalSummaryAdjuster {
                bounds: vec![90.0, 5112.0, 7531.0],
                rates: vec![1.19, 0.88, 0.94],
            }) as Rc<dyn FallbackApprovalSummaryModifier>,
            Rc::new(MinimumFallbackApprovalSummaryAdjuster { minimum: 38.3 }),
            Rc::new(CountryFallbackApprovalSummaryAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.89), ("retail".to_string(), 1.36)]),
            }),
            Rc::new(FlaggedFallbackApprovalSummaryAdjuster {
                flags: flags.clone(),
                flag: "fallback-approval-summary-rolling".to_string(),
                percent: 3.9,
            }),
            Rc::new(PercentFallbackApprovalSummaryAdjuster { percent: -23.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 94146.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_approval_summary_engine(&flags::new_registry());
        let item = new_fallback_approval_summary("code-997", "label-904");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
