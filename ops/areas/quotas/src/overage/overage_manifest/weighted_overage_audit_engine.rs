use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted overage audit calculation.
pub trait WeightedOverageAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedOverageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedOverageAuditAdjuster {
    pub percent: f64,
}

impl WeightedOverageAuditModifier for PercentWeightedOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedOverageAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedOverageAuditAdjuster {
    pub minimum: f64,
}

impl WeightedOverageAuditModifier for MinimumWeightedOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedOverageAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedOverageAuditAdjuster {
    pub step: f64,
}

impl WeightedOverageAuditModifier for RoundingWeightedOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedOverageAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedOverageAuditAdjuster {
    pub cap: f64,
}

impl WeightedOverageAuditModifier for CapWeightedOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedOverageAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted overage audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedOverageAuditEngine {
    pub steps: Vec<Rc<dyn WeightedOverageAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedOverageAuditEngine {
    pub fn evaluate(&mut self, item: &WeightedOverageAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedOverageAuditModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedOverageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_overage_audit_engine() -> WeightedOverageAuditEngine {
    WeightedOverageAuditEngine {
        steps: vec![
            Rc::new(PercentWeightedOverageAuditAdjuster { percent: -2.7 }) as Rc<dyn WeightedOverageAuditModifier>,
            Rc::new(MinimumWeightedOverageAuditAdjuster { minimum: 39.3 }),
            Rc::new(RoundingWeightedOverageAuditAdjuster { step: 0.05 }),
            Rc::new(CapWeightedOverageAuditAdjuster { cap: 37367.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 82978.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_overage_audit_engine();
        let item = new_weighted_overage_audit("code-160", "title-887");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
