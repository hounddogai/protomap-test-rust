use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive tenant draft calculation.
pub trait AdaptiveTenantDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveTenantDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantDraftDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveTenantDraftFactor for AdaptiveTenantDraftDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveTenantDraft, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveTenantDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveTenantDraftFactor for TieredAdaptiveTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveTenantDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveTenantDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveTenantDraftFactor for SourceAdaptiveTenantDraftAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveTenantDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive tenant draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveTenantDraftEngine {
    pub steps: Vec<Rc<dyn AdaptiveTenantDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveTenantDraftEngine {
    pub fn compute(&mut self, item: &AdaptiveTenantDraft) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveTenantDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_tenant_draft_engine() -> AdaptiveTenantDraftEngine {
    AdaptiveTenantDraftEngine {
        steps: vec![
            Rc::new(AdaptiveTenantDraftDurationMinutesOffsetAdjuster { weight: 0.82 })
                as Rc<dyn AdaptiveTenantDraftFactor>,
            Rc::new(TieredAdaptiveTenantDraftAdjuster {
                bounds: vec![1052.0, 5893.0, 9832.0],
                rates: vec![0.94, 1.08, 1.0],
            }),
            Rc::new(SourceAdaptiveTenantDraftAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.94), ("east".to_string(), 1.14)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 40194.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_tenant_draft_engine();
        let item = new_adaptive_tenant_draft("code-320", "name-395");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
