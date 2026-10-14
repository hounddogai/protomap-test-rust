use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive audit event score calculation.
pub trait AdaptiveAuditEventScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAuditEventScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveAuditEventScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveAuditEventScoreAdjuster for TieredAdaptiveAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveAuditEventScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveAuditEventScoreAdjuster {
    pub minimum: f64,
}

impl AdaptiveAuditEventScoreAdjuster for MinimumAdaptiveAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveAuditEventScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveAuditEventScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveAuditEventScoreAdjuster for SourceAdaptiveAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveAuditEventScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveAuditEventScoreAdjuster {
    pub percent: f64,
}

impl AdaptiveAuditEventScoreAdjuster for PercentAdaptiveAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveAuditEventScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive audit event score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventScoreEngine {
    pub steps: Vec<Rc<dyn AdaptiveAuditEventScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAuditEventScoreEngine {
    pub fn evaluate(&mut self, item: &AdaptiveAuditEventScore) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveAuditEventScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAuditEventScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_audit_event_score_engine() -> AdaptiveAuditEventScoreEngine {
    AdaptiveAuditEventScoreEngine {
        steps: vec![
            Rc::new(TieredAdaptiveAuditEventScoreAdjuster {
                bounds: vec![1601.0, 7580.0, 8036.0],
                rates: vec![0.92, 1.03, 0.97],
            }) as Rc<dyn AdaptiveAuditEventScoreAdjuster>,
            Rc::new(MinimumAdaptiveAuditEventScoreAdjuster { minimum: 30.2 }),
            Rc::new(SourceAdaptiveAuditEventScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.98), ("east".to_string(), 1.34)]),
            }),
            Rc::new(PercentAdaptiveAuditEventScoreAdjuster { percent: 2.4 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 51101.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_audit_event_score_engine();
        let item = new_adaptive_audit_event_score("code-938", "name-532");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
