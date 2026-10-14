use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled zone audit calculation.
pub trait ScheduledZoneAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceScheduledZoneAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledZoneAuditModifier for NamespaceScheduledZoneAuditAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledZoneAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledZoneAuditModifier for TieredScheduledZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledZoneAuditAdjuster {
    pub cap: f64,
}

impl ScheduledZoneAuditModifier for CapScheduledZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledZoneAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledZoneAuditModifier for FlaggedScheduledZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAuditDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledZoneAuditModifier for ScheduledZoneAuditDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledZoneAudit, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the scheduled zone audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledZoneAuditEngine {
    pub steps: Vec<Rc<dyn ScheduledZoneAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledZoneAuditEngine {
    pub fn evaluate(&mut self, item: &ScheduledZoneAudit) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledZoneAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_zone_audit_engine(flags: &flags::Registry) -> ScheduledZoneAuditEngine {
    ScheduledZoneAuditEngine {
        steps: vec![
            Rc::new(NamespaceScheduledZoneAuditAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.0)]),
            }) as Rc<dyn ScheduledZoneAuditModifier>,
            Rc::new(TieredScheduledZoneAuditAdjuster {
                bounds: vec![508.0, 6130.0, 9964.0],
                rates: vec![1.1, 1.07, 0.99],
            }),
            Rc::new(CapScheduledZoneAuditAdjuster { cap: 5356.0 }),
            Rc::new(FlaggedScheduledZoneAuditAdjuster {
                flags: flags.clone(),
                flag: "scheduled-zone-audit-default".to_string(),
                percent: -9.0,
            }),
            Rc::new(ScheduledZoneAuditDurationMinutesOffsetAdjuster { weight: 1.85 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 55119.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_zone_audit_engine(&flags::new_registry());
        let item = new_scheduled_zone_audit("code-338", "label-939");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
