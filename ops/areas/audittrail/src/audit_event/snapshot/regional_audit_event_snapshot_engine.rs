use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional audit event snapshot calculation.
pub trait RegionalAuditEventSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalAuditEventSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalAuditEventSnapshotAdjuster {
    pub step: f64,
}

impl RegionalAuditEventSnapshotFactor for RoundingRegionalAuditEventSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalAuditEventSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RegionalAuditEventSnapshotFactor for RegionalAuditEventSnapshotCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalAuditEventSnapshot, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalAuditEventSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalAuditEventSnapshotFactor for TitleRegionalAuditEventSnapshotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalAuditEventSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalAuditEventSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalAuditEventSnapshotFactor for FlaggedRegionalAuditEventSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalAuditEventSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional audit event snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalAuditEventSnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalAuditEventSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalAuditEventSnapshotEngine {
    pub fn price(&mut self, item: &RegionalAuditEventSnapshot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalAuditEventSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalAuditEventSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_audit_event_snapshot_engine(flags: &flags::Registry) -> RegionalAuditEventSnapshotEngine {
    RegionalAuditEventSnapshotEngine {
        steps: vec![
            Rc::new(RoundingRegionalAuditEventSnapshotAdjuster { step: 5.0 })
                as Rc<dyn RegionalAuditEventSnapshotFactor>,
            Rc::new(RegionalAuditEventSnapshotCeilingOffsetAdjuster { weight: 0.12 }),
            Rc::new(TitleRegionalAuditEventSnapshotAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.83)]),
            }),
            Rc::new(FlaggedRegionalAuditEventSnapshotAdjuster {
                flags: flags.clone(),
                flag: "regional-audit-event-snapshot-scheduled".to_string(),
                percent: -8.5,
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 87202.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_audit_event_snapshot_engine(&flags::new_registry());
        let item = new_regional_audit_event_snapshot("code-776", "title-134");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
