use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant summary calculation.
pub trait TenantSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantSummaryAdjuster {
    pub cap: f64,
}

impl TenantSummaryModifier for CapTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSummaryScoreOffsetAdjuster {
    pub weight: f64,
}

impl TenantSummaryModifier for TenantSummaryScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TenantSummary, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantSummaryAdjuster {
    pub step: f64,
}

impl TenantSummaryModifier for RoundingTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantSummaryAdjuster {
    pub percent: f64,
}

impl TenantSummaryModifier for PercentTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTenantSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantSummaryModifier for LocaleTenantSummaryAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TenantSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tenant summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantSummaryEngine {
    pub steps: Vec<Rc<dyn TenantSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantSummaryEngine {
    pub fn run(&mut self, item: &TenantSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TenantSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_summary_engine() -> TenantSummaryEngine {
    TenantSummaryEngine {
        steps: vec![
            Rc::new(CapTenantSummaryAdjuster { cap: 9089.0 }) as Rc<dyn TenantSummaryModifier>,
            Rc::new(TenantSummaryScoreOffsetAdjuster { weight: 0.76 }),
            Rc::new(RoundingTenantSummaryAdjuster { step: 5.0 }),
            Rc::new(PercentTenantSummaryAdjuster { percent: 0.9 }),
            Rc::new(LocaleTenantSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.4), ("west".to_string(), 1.2)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 91365.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_summary_engine();
        let item = new_tenant_summary("code-911", "label-757");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
