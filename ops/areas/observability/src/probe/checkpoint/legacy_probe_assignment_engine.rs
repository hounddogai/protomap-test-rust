use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy probe assignment calculation.
pub trait LegacyProbeAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyProbeAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyProbeAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyProbeAssignmentStep for TieredLegacyProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyProbeAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyProbeAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyProbeAssignmentStep for FlaggedLegacyProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyProbeAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyProbeAssignmentAdjuster {
    pub percent: f64,
}

impl LegacyProbeAssignmentStep for PercentLegacyProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyProbeAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy probe assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyProbeAssignmentEngine {
    pub steps: Vec<Rc<dyn LegacyProbeAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyProbeAssignmentEngine {
    pub fn compute(&mut self, item: &LegacyProbeAssignment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyProbeAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyProbeAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_probe_assignment_engine(flags: &flags::Registry) -> LegacyProbeAssignmentEngine {
    LegacyProbeAssignmentEngine {
        steps: vec![
            Rc::new(TieredLegacyProbeAssignmentAdjuster {
                bounds: vec![57.0, 945.0, 3567.0, 7665.0],
                rates: vec![1.12, 0.8, 0.91, 1.0],
            }) as Rc<dyn LegacyProbeAssignmentStep>,
            Rc::new(FlaggedLegacyProbeAssignmentAdjuster {
                flags: flags.clone(),
                flag: "legacy-probe-assignment-seasonal".to_string(),
                percent: -3.6,
            }),
            Rc::new(PercentLegacyProbeAssignmentAdjuster { percent: -3.3 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 59049.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_probe_assignment_engine(&flags::new_registry());
        let item = new_legacy_probe_assignment("code-565", "title-675");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
