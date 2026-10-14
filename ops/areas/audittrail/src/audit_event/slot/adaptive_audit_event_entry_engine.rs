use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive audit event entry calculation.
pub trait AdaptiveAuditEventEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdaptiveAuditEventEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveAuditEventEntryFactor for SegmentAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveAuditEventEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveAuditEventEntryFactor for TieredAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveAuditEventEntryAdjuster {
    pub minimum: f64,
}

impl AdaptiveAuditEventEntryFactor for MinimumAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAuditEventEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAuditEventEntryFactor for FlaggedAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAuditEventEntryAdjuster {
    pub cap: f64,
}

impl AdaptiveAuditEventEntryFactor for CapAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAuditEventEntryAdjuster {
    pub step: f64,
}

impl AdaptiveAuditEventEntryFactor for RoundingAdaptiveAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAuditEventEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive audit event entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAuditEventEntryEngine {
    pub steps: Vec<Rc<dyn AdaptiveAuditEventEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAuditEventEntryEngine {
    pub fn compute(&mut self, item: &AdaptiveAuditEventEntry) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveAuditEventEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAuditEventEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_audit_event_entry_engine(flags: &flags::Registry) -> AdaptiveAuditEventEntryEngine {
    AdaptiveAuditEventEntryEngine {
        steps: vec![
            Rc::new(SegmentAdaptiveAuditEventEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86)]),
            }) as Rc<dyn AdaptiveAuditEventEntryFactor>,
            Rc::new(TieredAdaptiveAuditEventEntryAdjuster {
                bounds: vec![3945.0, 7458.0],
                rates: vec![0.82, 1.12],
            }),
            Rc::new(MinimumAdaptiveAuditEventEntryAdjuster { minimum: 33.0 }),
            Rc::new(FlaggedAdaptiveAuditEventEntryAdjuster {
                flags: flags.clone(),
                flag: "adaptive-audit-event-entry-tiered".to_string(),
                percent: -1.7,
            }),
            Rc::new(CapAdaptiveAuditEventEntryAdjuster { cap: 38793.0 }),
            Rc::new(RoundingAdaptiveAuditEventEntryAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 47074.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_audit_event_entry_engine(&flags::new_registry());
        let item = new_adaptive_audit_event_entry("code-756", "label-538");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
