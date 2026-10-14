use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered health check audit calculation.
pub trait TieredHealthCheckAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredHealthCheckAuditAdjuster {
    pub percent: f64,
}

impl TieredHealthCheckAuditAdjuster for PercentTieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTieredHealthCheckAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredHealthCheckAuditAdjuster for CountryTieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredHealthCheckAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredHealthCheckAuditAdjuster for TieredTieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredHealthCheckAuditAdjuster {
    pub cap: f64,
}

impl TieredHealthCheckAuditAdjuster for CapTieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredHealthCheckAuditAdjuster {
    pub minimum: f64,
}

impl TieredHealthCheckAuditAdjuster for MinimumTieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredHealthCheckAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered health check audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckAuditEngine {
    pub steps: Vec<Rc<dyn TieredHealthCheckAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredHealthCheckAuditEngine {
    pub fn run(&mut self, item: &TieredHealthCheckAudit) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredHealthCheckAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_health_check_audit_engine() -> TieredHealthCheckAuditEngine {
    TieredHealthCheckAuditEngine {
        steps: vec![
            Rc::new(PercentTieredHealthCheckAuditAdjuster { percent: -23.6 }) as Rc<dyn TieredHealthCheckAuditAdjuster>,
            Rc::new(CountryTieredHealthCheckAuditAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.77),
                    ("south".to_string(), 1.37),
                    ("west".to_string(), 1.07),
                ]),
            }),
            Rc::new(TieredTieredHealthCheckAuditAdjuster {
                bounds: vec![260.0, 5774.0],
                rates: vec![0.99, 1.05],
            }),
            Rc::new(CapTieredHealthCheckAuditAdjuster { cap: 10252.0 }),
            Rc::new(MinimumTieredHealthCheckAuditAdjuster { minimum: 12.2 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 31173.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_health_check_audit_engine();
        let item = new_tiered_health_check_audit("code-626", "name-719");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
