use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo draft calculation.
pub trait SloDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloDraftAdjuster {
    pub minimum: f64,
}

impl SloDraftFactor for MinimumSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloDraftFactor for FlaggedSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl SloDraftFactor for SloDraftEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &SloDraft, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSloDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloDraftFactor for LabelSloDraftAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SloDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloDraftAdjuster {
    pub cap: f64,
}

impl SloDraftFactor for CapSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the slo draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloDraftEngine {
    pub steps: Vec<Rc<dyn SloDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloDraftEngine {
    pub fn run(&mut self, item: &SloDraft) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[SloDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_draft_engine(flags: &flags::Registry) -> SloDraftEngine {
    SloDraftEngine {
        steps: vec![
            Rc::new(MinimumSloDraftAdjuster { minimum: 26.0 }) as Rc<dyn SloDraftFactor>,
            Rc::new(FlaggedSloDraftAdjuster {
                flags: flags.clone(),
                flag: "slo-draft-regional".to_string(),
                percent: 4.3,
            }),
            Rc::new(SloDraftEndsAtOffsetAdjuster { weight: 0.63 }),
            Rc::new(LabelSloDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.31)]),
            }),
            Rc::new(CapSloDraftAdjuster { cap: 18178.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 95974.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_draft_engine(&flags::new_registry());
        let item = new_slo_draft("code-849", "label-69");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
