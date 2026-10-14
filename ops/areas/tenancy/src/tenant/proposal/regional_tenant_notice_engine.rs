use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional tenant notice calculation.
pub trait RegionalTenantNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalTenantNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalTenantNoticeAdjuster {
    pub minimum: f64,
}

impl RegionalTenantNoticeAdjuster for MinimumRegionalTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalTenantNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRegionalTenantNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalTenantNoticeAdjuster for NamespaceRegionalTenantNoticeAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RegionalTenantNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalTenantNoticeAdjuster {
    pub cap: f64,
}

impl RegionalTenantNoticeAdjuster for CapRegionalTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalTenantNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalTenantNoticeAdjuster {
    pub percent: f64,
}

impl RegionalTenantNoticeAdjuster for PercentRegionalTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalTenantNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional tenant notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalTenantNoticeEngine {
    pub steps: Vec<Rc<dyn RegionalTenantNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalTenantNoticeEngine {
    pub fn evaluate(&mut self, item: &RegionalTenantNotice) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalTenantNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_tenant_notice_engine() -> RegionalTenantNoticeEngine {
    RegionalTenantNoticeEngine {
        steps: vec![
            Rc::new(MinimumRegionalTenantNoticeAdjuster { minimum: 2.2 }) as Rc<dyn RegionalTenantNoticeAdjuster>,
            Rc::new(NamespaceRegionalTenantNoticeAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.86)]),
            }),
            Rc::new(CapRegionalTenantNoticeAdjuster { cap: 31905.0 }),
            Rc::new(PercentRegionalTenantNoticeAdjuster { percent: -13.4 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 96301.7,
    }
}
