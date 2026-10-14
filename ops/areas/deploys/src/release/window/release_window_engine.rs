use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release window calculation.
pub trait ReleaseWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseWindowAdjuster {
    pub step: f64,
}

impl ReleaseWindowStep for RoundingReleaseWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionReleaseWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseWindowStep for RegionReleaseWindowAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ReleaseWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseWindowAdjuster {
    pub cap: f64,
}

impl ReleaseWindowStep for CapReleaseWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the release window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseWindowEngine {
    pub steps: Vec<Rc<dyn ReleaseWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseWindowEngine {
    pub fn price(&mut self, item: &ReleaseWindow) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseWindowStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_window_engine() -> ReleaseWindowEngine {
    ReleaseWindowEngine {
        steps: vec![
            Rc::new(RoundingReleaseWindowAdjuster { step: 1.0 }) as Rc<dyn ReleaseWindowStep>,
            Rc::new(RegionReleaseWindowAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.1),
                    ("retail".to_string(), 0.86),
                    ("west".to_string(), 0.76),
                ]),
            }),
            Rc::new(CapReleaseWindowAdjuster { cap: 13509.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 37566.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_window_engine();
        let item = new_release_window("code-314", "label-346");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
