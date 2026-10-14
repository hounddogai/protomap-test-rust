use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension audit calculation.
pub trait SuspensionAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionAuditAdjuster for TieredSuspensionAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSuspensionAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionAuditAdjuster for RegionSuspensionAuditAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SuspensionAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionAuditAdjuster {
    pub step: f64,
}

impl SuspensionAuditAdjuster for RoundingSuspensionAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the suspension audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionAuditEngine {
    pub steps: Vec<Rc<dyn SuspensionAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionAuditEngine {
    pub fn run(&mut self, item: &SuspensionAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_suspension_audit_engine() -> SuspensionAuditEngine {
    SuspensionAuditEngine {
        steps: vec![
            Rc::new(TieredSuspensionAuditAdjuster {
                bounds: vec![2180.0, 4948.0, 8134.0],
                rates: vec![0.99, 1.0, 1.01],
            }) as Rc<dyn SuspensionAuditAdjuster>,
            Rc::new(RegionSuspensionAuditAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.77),
                    ("east".to_string(), 0.71),
                    ("north".to_string(), 0.85),
                ]),
            }),
            Rc::new(RoundingSuspensionAuditAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 18495.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_audit_engine();
        let item = new_suspension_audit("code-396", "name-680");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
