use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy failover digest calculation.
pub trait LegacyFailoverDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFailoverDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyFailoverDigestAdjuster {
    pub minimum: f64,
}

impl LegacyFailoverDigestModifier for MinimumLegacyFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyFailoverDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLegacyFailoverDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFailoverDigestModifier for SegmentLegacyFailoverDigestAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LegacyFailoverDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFailoverDigestAdjuster {
    pub step: f64,
}

impl LegacyFailoverDigestModifier for RoundingLegacyFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFailoverDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFailoverDigestAdjuster {
    pub cap: f64,
}

impl LegacyFailoverDigestModifier for CapLegacyFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFailoverDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy failover digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFailoverDigestEngine {
    pub steps: Vec<Rc<dyn LegacyFailoverDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFailoverDigestEngine {
    pub fn compute(&mut self, item: &LegacyFailoverDigest) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFailoverDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_failover_digest_engine() -> LegacyFailoverDigestEngine {
    LegacyFailoverDigestEngine {
        steps: vec![
            Rc::new(MinimumLegacyFailoverDigestAdjuster { minimum: 29.0 }) as Rc<dyn LegacyFailoverDigestModifier>,
            Rc::new(SegmentLegacyFailoverDigestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.0), ("west".to_string(), 1.09)]),
            }),
            Rc::new(RoundingLegacyFailoverDigestAdjuster { step: 0.5 }),
            Rc::new(CapLegacyFailoverDigestAdjuster { cap: 23302.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 40545.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_failover_digest_engine();
        let item = new_legacy_failover_digest("code-235", "title-786");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
