use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback escalation cycle calculation.
pub trait FallbackEscalationCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackEscalationCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackEscalationCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackEscalationCycleAdjuster for LabelFallbackEscalationCycleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackEscalationCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackEscalationCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackEscalationCycleAdjuster for FlaggedFallbackEscalationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackEscalationCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackEscalationCycleAdjuster {
    pub cap: f64,
}

impl FallbackEscalationCycleAdjuster for CapFallbackEscalationCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackEscalationCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback escalation cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackEscalationCycleEngine {
    pub steps: Vec<Rc<dyn FallbackEscalationCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackEscalationCycleEngine {
    pub fn evaluate(&mut self, item: &FallbackEscalationCycle) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackEscalationCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_escalation_cycle_engine(flags: &flags::Registry) -> FallbackEscalationCycleEngine {
    FallbackEscalationCycleEngine {
        steps: vec![
            Rc::new(LabelFallbackEscalationCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.09), ("retail".to_string(), 0.79)]),
            }) as Rc<dyn FallbackEscalationCycleAdjuster>,
            Rc::new(FlaggedFallbackEscalationCycleAdjuster {
                flags: flags.clone(),
                flag: "fallback-escalation-cycle-lenient".to_string(),
                percent: -1.2,
            }),
            Rc::new(CapFallbackEscalationCycleAdjuster { cap: 31110.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 1426.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_escalation_cycle_engine(&flags::new_registry());
        let item = new_fallback_escalation_cycle("code-172", "label-122");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
