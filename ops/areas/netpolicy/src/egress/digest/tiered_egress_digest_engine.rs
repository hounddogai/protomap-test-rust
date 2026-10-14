use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered egress digest calculation.
pub trait TieredEgressDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredEgressDigestAdjuster {
    pub percent: f64,
}

impl TieredEgressDigestAdjuster for PercentTieredEgressDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredEgressDigestAdjuster {
    pub cap: f64,
}

impl TieredEgressDigestAdjuster for CapTieredEgressDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredEgressDigestAdjuster {
    pub minimum: f64,
}

impl TieredEgressDigestAdjuster for MinimumTieredEgressDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl TieredEgressDigestAdjuster for TieredEgressDigestExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTieredEgressDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredEgressDigestAdjuster for NamespaceTieredEgressDigestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredEgressDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredEgressDigestAdjuster for FlaggedTieredEgressDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredEgressDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered egress digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredEgressDigestEngine {
    pub steps: Vec<Rc<dyn TieredEgressDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredEgressDigestEngine {
    pub fn evaluate(&mut self, item: &TieredEgressDigest) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[TieredEgressDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_egress_digest_engine(flags: &flags::Registry) -> TieredEgressDigestEngine {
    TieredEgressDigestEngine {
        steps: vec![
            Rc::new(PercentTieredEgressDigestAdjuster { percent: -10.4 }) as Rc<dyn TieredEgressDigestAdjuster>,
            Rc::new(CapTieredEgressDigestAdjuster { cap: 35921.0 }),
            Rc::new(MinimumTieredEgressDigestAdjuster { minimum: 10.6 }),
            Rc::new(TieredEgressDigestExpiresAtOffsetAdjuster { weight: 0.49 }),
            Rc::new(NamespaceTieredEgressDigestAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.15),
                    ("online".to_string(), 1.31),
                    ("east".to_string(), 1.07),
                ]),
            }),
            Rc::new(FlaggedTieredEgressDigestAdjuster {
                flags: flags.clone(),
                flag: "tiered-egress-digest-adaptive".to_string(),
                percent: 3.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 76930.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_egress_digest_engine(&flags::new_registry());
        let item = new_tiered_egress_digest("code-264", "name-311");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
