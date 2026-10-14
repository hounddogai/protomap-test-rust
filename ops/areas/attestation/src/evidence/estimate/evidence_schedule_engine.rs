use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence schedule calculation.
pub trait EvidenceScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterEvidenceScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceScheduleAdjuster for ClusterEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceScheduleAdjuster for FlaggedEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceScheduleAdjuster for TieredEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceScheduleAdjuster {
    pub percent: f64,
}

impl EvidenceScheduleAdjuster for PercentEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceScheduleAdjuster {
    pub minimum: f64,
}

impl EvidenceScheduleAdjuster for MinimumEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceScheduleAdjuster {
    pub step: f64,
}

impl EvidenceScheduleAdjuster for RoundingEvidenceScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the evidence schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceScheduleEngine {
    pub steps: Vec<Rc<dyn EvidenceScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceScheduleEngine {
    pub fn evaluate(&mut self, item: &EvidenceSchedule) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_schedule_engine(flags: &flags::Registry) -> EvidenceScheduleEngine {
    EvidenceScheduleEngine {
        steps: vec![
            Rc::new(ClusterEvidenceScheduleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.03)]),
            }) as Rc<dyn EvidenceScheduleAdjuster>,
            Rc::new(FlaggedEvidenceScheduleAdjuster {
                flags: flags.clone(),
                flag: "evidence-schedule-lenient".to_string(),
                percent: -1.3,
            }),
            Rc::new(TieredEvidenceScheduleAdjuster {
                bounds: vec![6880.0, 7125.0, 8053.0, 8116.0],
                rates: vec![1.12, 0.83, 0.86, 0.9],
            }),
            Rc::new(PercentEvidenceScheduleAdjuster { percent: 13.2 }),
            Rc::new(MinimumEvidenceScheduleAdjuster { minimum: 14.0 }),
            Rc::new(RoundingEvidenceScheduleAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 11641.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_schedule_engine(&flags::new_registry());
        let item = new_evidence_schedule("code-699", "label-602");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
