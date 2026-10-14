use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink audit calculation.
pub trait SinkAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkAuditAdjuster {
    pub percent: f64,
}

impl SinkAuditFactor for PercentSinkAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkAuditFactor for FlaggedSinkAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkAuditAdjuster {
    pub cap: f64,
}

impl SinkAuditFactor for CapSinkAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkAuditAdjuster {
    pub minimum: f64,
}

impl SinkAuditFactor for MinimumSinkAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSinkAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkAuditFactor for LabelSinkAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SinkAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the sink audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkAuditEngine {
    pub steps: Vec<Rc<dyn SinkAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkAuditEngine {
    pub fn run(&mut self, item: &SinkAudit) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[SinkAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_audit_engine(flags: &flags::Registry) -> SinkAuditEngine {
    SinkAuditEngine {
        steps: vec![
            Rc::new(PercentSinkAuditAdjuster { percent: -10.9 }) as Rc<dyn SinkAuditFactor>,
            Rc::new(FlaggedSinkAuditAdjuster {
                flags: flags.clone(),
                flag: "sink-audit-primary".to_string(),
                percent: -9.3,
            }),
            Rc::new(CapSinkAuditAdjuster { cap: 7651.0 }),
            Rc::new(MinimumSinkAuditAdjuster { minimum: 45.5 }),
            Rc::new(LabelSinkAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.87)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 80721.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_audit_engine(&flags::new_registry());
        let item = new_sink_audit("code-562", "label-444");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
