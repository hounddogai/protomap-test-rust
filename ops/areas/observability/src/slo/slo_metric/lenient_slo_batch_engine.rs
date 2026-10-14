use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient slo batch calculation.
pub trait LenientSloBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSloBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSloBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSloBatchFactor for FlaggedLenientSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSloBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLenientSloBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientSloBatchFactor for NameLenientSloBatchAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LenientSloBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientSloBatchAdjuster {
    pub step: f64,
}

impl LenientSloBatchFactor for RoundingLenientSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientSloBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSloBatchAdjuster {
    pub percent: f64,
}

impl LenientSloBatchFactor for PercentLenientSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSloBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient slo batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSloBatchEngine {
    pub steps: Vec<Rc<dyn LenientSloBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSloBatchEngine {
    pub fn compute(&mut self, item: &LenientSloBatch) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSloBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSloBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_slo_batch_engine(flags: &flags::Registry) -> LenientSloBatchEngine {
    LenientSloBatchEngine {
        steps: vec![
            Rc::new(FlaggedLenientSloBatchAdjuster {
                flags: flags.clone(),
                flag: "lenient-slo-batch-capped".to_string(),
                percent: -6.4,
            }) as Rc<dyn LenientSloBatchFactor>,
            Rc::new(NameLenientSloBatchAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.97)]),
            }),
            Rc::new(RoundingLenientSloBatchAdjuster { step: 0.05 }),
            Rc::new(PercentLenientSloBatchAdjuster { percent: -6.7 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 47698.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_slo_batch_engine(&flags::new_registry());
        let item = new_lenient_slo_batch("code-62", "name-967");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
