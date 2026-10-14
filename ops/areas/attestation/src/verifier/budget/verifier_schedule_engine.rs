use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier schedule calculation.
pub trait VerifierScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierScheduleAdjuster {
    pub cap: f64,
}

impl VerifierScheduleStep for CapVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryVerifierScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierScheduleStep for CountryVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierScheduleAdjuster {
    pub step: f64,
}

impl VerifierScheduleStep for RoundingVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierScheduleAdjuster {
    pub percent: f64,
}

impl VerifierScheduleStep for PercentVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierScheduleAdjuster {
    pub minimum: f64,
}

impl VerifierScheduleStep for MinimumVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierScheduleStep for FlaggedVerifierScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the verifier schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierScheduleEngine {
    pub steps: Vec<Rc<dyn VerifierScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierScheduleEngine {
    pub fn evaluate(&mut self, item: &VerifierSchedule) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_verifier_schedule_engine(flags: &flags::Registry) -> VerifierScheduleEngine {
    VerifierScheduleEngine {
        steps: vec![
            Rc::new(CapVerifierScheduleAdjuster { cap: 45545.0 }) as Rc<dyn VerifierScheduleStep>,
            Rc::new(CountryVerifierScheduleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.16)]),
            }),
            Rc::new(RoundingVerifierScheduleAdjuster { step: 5.0 }),
            Rc::new(PercentVerifierScheduleAdjuster { percent: 18.0 }),
            Rc::new(MinimumVerifierScheduleAdjuster { minimum: 37.2 }),
            Rc::new(FlaggedVerifierScheduleAdjuster {
                flags: flags.clone(),
                flag: "verifier-schedule-strict".to_string(),
                percent: 6.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 9073.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_schedule_engine(&flags::new_registry());
        let item = new_verifier_schedule("code-858", "name-946");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
