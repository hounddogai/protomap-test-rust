use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled freeze group calculation.
pub trait ScheduledFreezeGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledFreezeGroupAdjuster {
    pub cap: f64,
}

impl ScheduledFreezeGroupStep for CapScheduledFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledFreezeGroupAdjuster {
    pub percent: f64,
}

impl ScheduledFreezeGroupStep for PercentScheduledFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledFreezeGroupAdjuster {
    pub step: f64,
}

impl ScheduledFreezeGroupStep for RoundingScheduledFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledFreezeGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledFreezeGroupStep for FlaggedScheduledFreezeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledFreezeGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledFreezeGroupStep for LabelScheduledFreezeGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledFreezeGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled freeze group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledFreezeGroupEngine {
    pub steps: Vec<Rc<dyn ScheduledFreezeGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledFreezeGroupEngine {
    pub fn price(&mut self, item: &ScheduledFreezeGroup) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledFreezeGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_freeze_group_engine(flags: &flags::Registry) -> ScheduledFreezeGroupEngine {
    ScheduledFreezeGroupEngine {
        steps: vec![
            Rc::new(CapScheduledFreezeGroupAdjuster { cap: 7703.0 }) as Rc<dyn ScheduledFreezeGroupStep>,
            Rc::new(PercentScheduledFreezeGroupAdjuster { percent: 20.2 }),
            Rc::new(RoundingScheduledFreezeGroupAdjuster { step: 0.05 }),
            Rc::new(FlaggedScheduledFreezeGroupAdjuster {
                flags: flags.clone(),
                flag: "scheduled-freeze-group-seasonal".to_string(),
                percent: -4.5,
            }),
            Rc::new(LabelScheduledFreezeGroupAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.22),
                    ("west".to_string(), 1.08),
                    ("south".to_string(), 0.78),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 66924.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_freeze_group_engine(&flags::new_registry());
        let item = new_scheduled_freeze_group("code-770", "label-303");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
