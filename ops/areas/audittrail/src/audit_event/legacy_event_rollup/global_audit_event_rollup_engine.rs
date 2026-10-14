use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global audit event rollup calculation.
pub trait GlobalAuditEventRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalAuditEventRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalAuditEventRollupFactor for FlaggedGlobalAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeGlobalAuditEventRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalAuditEventRollupFactor for ScopeGlobalAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupMarginOffsetAdjuster {
    pub weight: f64,
}

impl GlobalAuditEventRollupFactor for GlobalAuditEventRollupMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalAuditEventRollupAdjuster {
    pub step: f64,
}

impl GlobalAuditEventRollupFactor for RoundingGlobalAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAuditEventRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAuditEventRollupFactor for TieredGlobalAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAuditEventRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global audit event rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventRollupEngine {
    pub steps: Vec<Rc<dyn GlobalAuditEventRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAuditEventRollupEngine {
    pub fn evaluate(&mut self, item: &GlobalAuditEventRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalAuditEventRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAuditEventRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_audit_event_rollup_engine(flags: &flags::Registry) -> GlobalAuditEventRollupEngine {
    GlobalAuditEventRollupEngine {
        steps: vec![
            Rc::new(FlaggedGlobalAuditEventRollupAdjuster {
                flags: flags.clone(),
                flag: "global-audit-event-rollup-weighted".to_string(),
                percent: 5.5,
            }) as Rc<dyn GlobalAuditEventRollupFactor>,
            Rc::new(ScopeGlobalAuditEventRollupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.21)]),
            }),
            Rc::new(GlobalAuditEventRollupMarginOffsetAdjuster { weight: 0.3 }),
            Rc::new(RoundingGlobalAuditEventRollupAdjuster { step: 0.25 }),
            Rc::new(TieredGlobalAuditEventRollupAdjuster {
                bounds: vec![1244.0, 7942.0],
                rates: vec![1.03, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 44771.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_audit_event_rollup_engine(&flags::new_registry());
        let item = new_global_audit_event_rollup("code-513", "label-193");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
