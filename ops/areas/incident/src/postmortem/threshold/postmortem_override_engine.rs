use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem override calculation.
pub trait PostmortemOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemOverrideAdjuster {
    pub cap: f64,
}

impl PostmortemOverrideStep for CapPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemOverrideStep for FlaggedPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPostmortemOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemOverrideStep for ClusterPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PostmortemOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the postmortem override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemOverrideEngine {
    pub steps: Vec<Rc<dyn PostmortemOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemOverrideEngine {
    pub fn evaluate(&mut self, item: &PostmortemOverride) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_override_engine(flags: &flags::Registry) -> PostmortemOverrideEngine {
    PostmortemOverrideEngine {
        steps: vec![
            Rc::new(CapPostmortemOverrideAdjuster { cap: 1156.0 }) as Rc<dyn PostmortemOverrideStep>,
            Rc::new(FlaggedPostmortemOverrideAdjuster {
                flags: flags.clone(),
                flag: "postmortem-override-tiered".to_string(),
                percent: 5.5,
            }),
            Rc::new(ClusterPostmortemOverrideAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.1),
                    ("online".to_string(), 1.14),
                    ("retail".to_string(), 0.74),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 80935.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_override_engine(&flags::new_registry());
        let item = new_postmortem_override("code-361", "label-759");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
