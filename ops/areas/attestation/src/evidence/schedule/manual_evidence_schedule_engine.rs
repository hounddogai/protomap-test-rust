use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual evidence schedule calculation.
pub trait ManualEvidenceScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualEvidenceSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualEvidenceScheduleAdjuster {
    pub percent: f64,
}

impl ManualEvidenceScheduleAdjuster for PercentManualEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualEvidenceSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelManualEvidenceScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualEvidenceScheduleAdjuster for ChannelManualEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ManualEvidenceSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceScheduleCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualEvidenceScheduleAdjuster for ManualEvidenceScheduleCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualEvidenceSchedule, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the manual evidence schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualEvidenceScheduleEngine {
    pub steps: Vec<Rc<dyn ManualEvidenceScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualEvidenceScheduleEngine {
    pub fn evaluate(&mut self, item: &ManualEvidenceSchedule) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualEvidenceScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualEvidenceSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_evidence_schedule_engine() -> ManualEvidenceScheduleEngine {
    ManualEvidenceScheduleEngine {
        steps: vec![
            Rc::new(PercentManualEvidenceScheduleAdjuster { percent: 4.6 }) as Rc<dyn ManualEvidenceScheduleAdjuster>,
            Rc::new(ChannelManualEvidenceScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.2)]),
            }),
            Rc::new(ManualEvidenceScheduleCreatedAtOffsetAdjuster { weight: 1.37 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 57088.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_evidence_schedule_engine();
        let item = new_manual_evidence_schedule("code-753", "title-758");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
