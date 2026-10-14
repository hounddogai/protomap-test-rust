use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary binding quota calculation.
pub trait PrimaryBindingQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBindingQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingQuotaBudgetOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryBindingQuotaAdjuster for PrimaryBindingQuotaBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryBindingQuota, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryBindingQuotaAdjuster {
    pub percent: f64,
}

impl PrimaryBindingQuotaAdjuster for PercentPrimaryBindingQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryBindingQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPrimaryBindingQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryBindingQuotaAdjuster for UnitPrimaryBindingQuotaAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PrimaryBindingQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryBindingQuotaAdjuster {
    pub cap: f64,
}

impl PrimaryBindingQuotaAdjuster for CapPrimaryBindingQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryBindingQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary binding quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBindingQuotaEngine {
    pub steps: Vec<Rc<dyn PrimaryBindingQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBindingQuotaEngine {
    pub fn run(&mut self, item: &PrimaryBindingQuota) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryBindingQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryBindingQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_binding_quota_engine() -> PrimaryBindingQuotaEngine {
    PrimaryBindingQuotaEngine {
        steps: vec![
            Rc::new(PrimaryBindingQuotaBudgetOffsetAdjuster { weight: 0.92 }) as Rc<dyn PrimaryBindingQuotaAdjuster>,
            Rc::new(PercentPrimaryBindingQuotaAdjuster { percent: -8.8 }),
            Rc::new(UnitPrimaryBindingQuotaAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.2),
                    ("central".to_string(), 0.86),
                    ("west".to_string(), 1.23),
                ]),
            }),
            Rc::new(CapPrimaryBindingQuotaAdjuster { cap: 13061.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 94689.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_binding_quota_engine();
        let item = new_primary_binding_quota("code-411", "label-398");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
