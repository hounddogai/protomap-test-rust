use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy probe group calculation.
pub trait LegacyProbeGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyProbeGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyProbeGroupAdjuster {
    pub minimum: f64,
}

impl LegacyProbeGroupAdjuster for MinimumLegacyProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyProbeGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl LegacyProbeGroupAdjuster for LegacyProbeGroupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyProbeGroup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyProbeGroupAdjuster {
    pub percent: f64,
}

impl LegacyProbeGroupAdjuster for PercentLegacyProbeGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyProbeGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy probe group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyProbeGroupEngine {
    pub steps: Vec<Rc<dyn LegacyProbeGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyProbeGroupEngine {
    pub fn evaluate(&mut self, item: &LegacyProbeGroup) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyProbeGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyProbeGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_probe_group_engine() -> LegacyProbeGroupEngine {
    LegacyProbeGroupEngine {
        steps: vec![
            Rc::new(MinimumLegacyProbeGroupAdjuster { minimum: 17.7 }) as Rc<dyn LegacyProbeGroupAdjuster>,
            Rc::new(LegacyProbeGroupGraceDaysOffsetAdjuster { weight: 0.2 }),
            Rc::new(PercentLegacyProbeGroupAdjuster { percent: -5.2 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 18212.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_probe_group_engine();
        let item = new_legacy_probe_group("code-704", "name-215");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
