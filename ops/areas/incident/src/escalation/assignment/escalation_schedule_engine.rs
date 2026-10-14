use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation schedule calculation.
pub trait EscalationScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationScheduleModifier for TieredEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationScheduleAdjuster {
    pub step: f64,
}

impl EscalationScheduleModifier for RoundingEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEscalationScheduleAdjuster {
    pub percent: f64,
}

impl EscalationScheduleModifier for PercentEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationScheduleAdjuster {
    pub cap: f64,
}

impl EscalationScheduleModifier for CapEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEscalationScheduleAdjuster {
    pub minimum: f64,
}

impl EscalationScheduleModifier for MinimumEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationScheduleModifier for FlaggedEscalationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationScheduleEngine {
    pub steps: Vec<Rc<dyn EscalationScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationScheduleEngine {
    pub fn evaluate(&mut self, item: &EscalationSchedule) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationScheduleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_escalation_schedule_engine(flags: &flags::Registry) -> EscalationScheduleEngine {
    EscalationScheduleEngine {
        steps: vec![
            Rc::new(TieredEscalationScheduleAdjuster {
                bounds: vec![4992.0, 5566.0],
                rates: vec![0.95, 0.95],
            }) as Rc<dyn EscalationScheduleModifier>,
            Rc::new(RoundingEscalationScheduleAdjuster { step: 0.1 }),
            Rc::new(PercentEscalationScheduleAdjuster { percent: -8.3 }),
            Rc::new(CapEscalationScheduleAdjuster { cap: 12486.0 }),
            Rc::new(MinimumEscalationScheduleAdjuster { minimum: 42.2 }),
            Rc::new(FlaggedEscalationScheduleAdjuster {
                flags: flags.clone(),
                flag: "escalation-schedule-pending".to_string(),
                percent: -9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 37571.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_schedule_engine(&flags::new_registry());
        let item = new_escalation_schedule("code-121", "title-613");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
