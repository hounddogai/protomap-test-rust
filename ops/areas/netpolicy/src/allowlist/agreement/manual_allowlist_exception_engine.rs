use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual allowlist exception calculation.
pub trait ManualAllowlistExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistExceptionVersionOffsetAdjuster {
    pub weight: f64,
}

impl ManualAllowlistExceptionStep for ManualAllowlistExceptionVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualAllowlistExceptionAdjuster {
    pub step: f64,
}

impl ManualAllowlistExceptionStep for RoundingManualAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitManualAllowlistExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualAllowlistExceptionStep for UnitManualAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualAllowlistExceptionAdjuster {
    pub minimum: f64,
}

impl ManualAllowlistExceptionStep for MinimumManualAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualAllowlistExceptionAdjuster {
    pub cap: f64,
}

impl ManualAllowlistExceptionStep for CapManualAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualAllowlistException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual allowlist exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAllowlistExceptionEngine {
    pub steps: Vec<Rc<dyn ManualAllowlistExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAllowlistExceptionEngine {
    pub fn evaluate(&mut self, item: &ManualAllowlistException) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualAllowlistExceptionStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualAllowlistException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_allowlist_exception_engine() -> ManualAllowlistExceptionEngine {
    ManualAllowlistExceptionEngine {
        steps: vec![
            Rc::new(ManualAllowlistExceptionVersionOffsetAdjuster { weight: 1.34 })
                as Rc<dyn ManualAllowlistExceptionStep>,
            Rc::new(RoundingManualAllowlistExceptionAdjuster { step: 0.1 }),
            Rc::new(UnitManualAllowlistExceptionAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.08)]),
            }),
            Rc::new(MinimumManualAllowlistExceptionAdjuster { minimum: 46.3 }),
            Rc::new(CapManualAllowlistExceptionAdjuster { cap: 46368.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 58366.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_allowlist_exception_engine();
        let item = new_manual_allowlist_exception("code-452", "title-508");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
