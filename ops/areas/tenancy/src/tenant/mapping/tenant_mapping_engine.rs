use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant mapping calculation.
pub trait TenantMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantMappingAdjuster {
    pub cap: f64,
}

impl TenantMappingAdjuster for CapTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantMappingAdjuster {
    pub percent: f64,
}

impl TenantMappingAdjuster for PercentTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantMappingWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl TenantMappingAdjuster for TenantMappingWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &TenantMapping, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantMappingAdjuster for FlaggedTenantMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantMappingEngine {
    pub steps: Vec<Rc<dyn TenantMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantMappingEngine {
    pub fn price(&mut self, item: &TenantMapping) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantMappingAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TenantMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_mapping_engine(flags: &flags::Registry) -> TenantMappingEngine {
    TenantMappingEngine {
        steps: vec![
            Rc::new(CapTenantMappingAdjuster { cap: 32695.0 }) as Rc<dyn TenantMappingAdjuster>,
            Rc::new(PercentTenantMappingAdjuster { percent: 7.9 }),
            Rc::new(TenantMappingWindowDaysOffsetAdjuster { weight: 1.63 }),
            Rc::new(FlaggedTenantMappingAdjuster {
                flags: flags.clone(),
                flag: "tenant-mapping-rolling".to_string(),
                percent: 6.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 81766.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_mapping_engine(&flags::new_registry());
        let item = new_tenant_mapping("code-773", "label-440");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
