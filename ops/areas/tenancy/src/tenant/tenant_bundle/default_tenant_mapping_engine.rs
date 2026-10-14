use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default tenant mapping calculation.
pub trait DefaultTenantMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTenantMappingAdjuster {
    pub cap: f64,
}

impl DefaultTenantMappingModifier for CapDefaultTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultTenantMappingAdjuster {
    pub step: f64,
}

impl DefaultTenantMappingModifier for RoundingDefaultTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultTenantMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultTenantMappingModifier for TieredDefaultTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDefaultTenantMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultTenantMappingModifier for ClusterDefaultTenantMappingAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultTenantMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultTenantMappingModifier for FlaggedDefaultTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultTenantMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default tenant mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTenantMappingEngine {
    pub steps: Vec<Rc<dyn DefaultTenantMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTenantMappingEngine {
    pub fn compute(&mut self, item: &DefaultTenantMapping) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTenantMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_tenant_mapping_engine(flags: &flags::Registry) -> DefaultTenantMappingEngine {
    DefaultTenantMappingEngine {
        steps: vec![
            Rc::new(CapDefaultTenantMappingAdjuster { cap: 2764.0 }) as Rc<dyn DefaultTenantMappingModifier>,
            Rc::new(RoundingDefaultTenantMappingAdjuster { step: 1.0 }),
            Rc::new(TieredDefaultTenantMappingAdjuster {
                bounds: vec![1772.0, 5126.0, 9866.0],
                rates: vec![0.88, 0.89, 1.07],
            }),
            Rc::new(ClusterDefaultTenantMappingAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.73)]),
            }),
            Rc::new(FlaggedDefaultTenantMappingAdjuster {
                flags: flags.clone(),
                flag: "default-tenant-mapping-tiered".to_string(),
                percent: -4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 51664.8,
    }
}
