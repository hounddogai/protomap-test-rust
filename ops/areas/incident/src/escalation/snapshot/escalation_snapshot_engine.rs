use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the escalation snapshot calculation.
pub trait EscalationSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationSnapshotStep for FlaggedEscalationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationSnapshotStep for TieredEscalationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationSnapshotAdjuster {
    pub percent: f64,
}

impl EscalationSnapshotStep for PercentEscalationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationSnapshotAdjuster {
    pub minimum: f64,
}

impl EscalationSnapshotStep for MinimumEscalationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelEscalationSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationSnapshotStep for ChannelEscalationSnapshotAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &EscalationSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the escalation snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationSnapshotEngine {
    pub steps: Vec<Rc<dyn EscalationSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationSnapshotEngine {
    pub fn evaluate(&mut self, item: &EscalationSnapshot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_snapshot_engine(flags: &flags::Registry) -> EscalationSnapshotEngine {
    EscalationSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedEscalationSnapshotAdjuster {
                flags: flags.clone(),
                flag: "escalation-snapshot-lenient".to_string(),
                percent: 8.2,
            }) as Rc<dyn EscalationSnapshotStep>,
            Rc::new(TieredEscalationSnapshotAdjuster {
                bounds: vec![3530.0, 4400.0, 9182.0],
                rates: vec![1.01, 1.13, 1.16],
            }),
            Rc::new(PercentEscalationSnapshotAdjuster { percent: -11.3 }),
            Rc::new(MinimumEscalationSnapshotAdjuster { minimum: 31.4 }),
            Rc::new(ChannelEscalationSnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.79), ("west".to_string(), 0.77)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 16932.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_snapshot_engine(&flags::new_registry());
        let item = new_escalation_snapshot("code-220", "label-609");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
