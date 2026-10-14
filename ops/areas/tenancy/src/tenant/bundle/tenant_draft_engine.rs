use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant draft calculation.
pub trait TenantDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantDraftAdjuster {
    pub percent: f64,
}

impl TenantDraftModifier for PercentTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTenantDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantDraftModifier for RegionTenantDraftAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantDraftAdjuster {
    pub step: f64,
}

impl TenantDraftModifier for RoundingTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantDraftAdjuster {
    pub minimum: f64,
}

impl TenantDraftModifier for MinimumTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantDraftAdjuster {
    pub cap: f64,
}

impl TenantDraftModifier for CapTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftBurstOffsetAdjuster {
    pub weight: f64,
}

impl TenantDraftModifier for TenantDraftBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &TenantDraft, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the tenant draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantDraftEngine {
    pub steps: Vec<Rc<dyn TenantDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantDraftEngine {
    pub fn price(&mut self, item: &TenantDraft) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TenantDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_draft_engine() -> TenantDraftEngine {
    TenantDraftEngine {
        steps: vec![
            Rc::new(PercentTenantDraftAdjuster { percent: 11.9 }) as Rc<dyn TenantDraftModifier>,
            Rc::new(RegionTenantDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.15), ("west".to_string(), 0.83)]),
            }),
            Rc::new(RoundingTenantDraftAdjuster { step: 0.25 }),
            Rc::new(MinimumTenantDraftAdjuster { minimum: 27.3 }),
            Rc::new(CapTenantDraftAdjuster { cap: 6256.0 }),
            Rc::new(TenantDraftBurstOffsetAdjuster { weight: 0.91 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 78168.8,
    }
}
