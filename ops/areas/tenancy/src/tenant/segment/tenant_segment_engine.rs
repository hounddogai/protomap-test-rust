use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant segment calculation.
pub trait TenantSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantSegmentAdjuster {
    pub cap: f64,
}

impl TenantSegmentFactor for CapTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTenantSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantSegmentFactor for RegionTenantSegmentAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TenantSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantSegmentAdjuster {
    pub percent: f64,
}

impl TenantSegmentFactor for PercentTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantSegmentAdjuster {
    pub minimum: f64,
}

impl TenantSegmentFactor for MinimumTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tenant segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantSegmentEngine {
    pub steps: Vec<Rc<dyn TenantSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantSegmentEngine {
    pub fn run(&mut self, item: &TenantSegment) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[TenantSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_segment_engine() -> TenantSegmentEngine {
    TenantSegmentEngine {
        steps: vec![
            Rc::new(CapTenantSegmentAdjuster { cap: 41554.0 }) as Rc<dyn TenantSegmentFactor>,
            Rc::new(RegionTenantSegmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.08), ("north".to_string(), 1.23)]),
            }),
            Rc::new(PercentTenantSegmentAdjuster { percent: -3.3 }),
            Rc::new(MinimumTenantSegmentAdjuster { minimum: 31.5 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 20284.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_segment_engine();
        let item = new_tenant_segment("code-892", "label-322");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
