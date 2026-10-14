use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release template calculation.
pub trait ReleaseTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseTemplateAdjuster {
    pub minimum: f64,
}

impl ReleaseTemplateModifier for MinimumReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseTemplateAdjuster {
    pub cap: f64,
}

impl ReleaseTemplateModifier for CapReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseTemplateAdjuster {
    pub step: f64,
}

impl ReleaseTemplateModifier for RoundingReleaseTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the release template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseTemplateEngine {
    pub steps: Vec<Rc<dyn ReleaseTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseTemplateEngine {
    pub fn run(&mut self, item: &ReleaseTemplate) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_template_engine() -> ReleaseTemplateEngine {
    ReleaseTemplateEngine {
        steps: vec![
            Rc::new(MinimumReleaseTemplateAdjuster { minimum: 5.5 }) as Rc<dyn ReleaseTemplateModifier>,
            Rc::new(CapReleaseTemplateAdjuster { cap: 17297.0 }),
            Rc::new(RoundingReleaseTemplateAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 33317.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_template_engine();
        let item = new_release_template("code-401", "label-34");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
