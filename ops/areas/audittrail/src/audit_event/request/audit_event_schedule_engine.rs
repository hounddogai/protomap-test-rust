use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event schedule calculation.
pub trait AuditEventScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventScheduleFactor for TieredAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventScheduleAdjuster {
    pub step: f64,
}

impl AuditEventScheduleFactor for RoundingAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScheduleUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventScheduleFactor for AuditEventScheduleUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventScheduleAdjuster {
    pub percent: f64,
}

impl AuditEventScheduleFactor for PercentAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventScheduleAdjuster {
    pub minimum: f64,
}

impl AuditEventScheduleFactor for MinimumAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventScheduleAdjuster {
    pub cap: f64,
}

impl AuditEventScheduleFactor for CapAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the audit event schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventScheduleEngine {
    pub steps: Vec<Rc<dyn AuditEventScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventScheduleEngine {
    pub fn compute(&mut self, item: &AuditEventSchedule) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_schedule_engine() -> AuditEventScheduleEngine {
    AuditEventScheduleEngine {
        steps: vec![
            Rc::new(TieredAuditEventScheduleAdjuster {
                bounds: vec![6768.0, 6815.0, 9181.0],
                rates: vec![1.2, 0.95, 0.82],
            }) as Rc<dyn AuditEventScheduleFactor>,
            Rc::new(RoundingAuditEventScheduleAdjuster { step: 1.0 }),
            Rc::new(AuditEventScheduleUpdatedAtOffsetAdjuster { weight: 1.14 }),
            Rc::new(PercentAuditEventScheduleAdjuster { percent: 3.0 }),
            Rc::new(MinimumAuditEventScheduleAdjuster { minimum: 24.3 }),
            Rc::new(CapAuditEventScheduleAdjuster { cap: 21733.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 31950.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_schedule_engine();
        let item = new_audit_event_schedule("code-390", "name-732");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
