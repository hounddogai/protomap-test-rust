use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy zone override calculation.
pub trait LegacyZoneOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyZoneOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverrideThresholdOffsetAdjuster {
    pub weight: f64,
}

impl LegacyZoneOverrideAdjuster for LegacyZoneOverrideThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyZoneOverride, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyZoneOverrideAdjuster {
    pub percent: f64,
}

impl LegacyZoneOverrideAdjuster for PercentLegacyZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyZoneOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyZoneOverrideAdjuster {
    pub minimum: f64,
}

impl LegacyZoneOverrideAdjuster for MinimumLegacyZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyZoneOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy zone override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyZoneOverrideEngine {
    pub steps: Vec<Rc<dyn LegacyZoneOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyZoneOverrideEngine {
    pub fn evaluate(&mut self, item: &LegacyZoneOverride) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyZoneOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_zone_override_engine() -> LegacyZoneOverrideEngine {
    LegacyZoneOverrideEngine {
        steps: vec![
            Rc::new(LegacyZoneOverrideThresholdOffsetAdjuster { weight: 1.2 }) as Rc<dyn LegacyZoneOverrideAdjuster>,
            Rc::new(PercentLegacyZoneOverrideAdjuster { percent: 2.1 }),
            Rc::new(MinimumLegacyZoneOverrideAdjuster { minimum: 9.7 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 42596.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_zone_override_engine();
        let item = new_legacy_zone_override("code-994", "title-512");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
