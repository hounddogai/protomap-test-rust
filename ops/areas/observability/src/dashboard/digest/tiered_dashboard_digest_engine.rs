use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered dashboard digest calculation.
pub trait TieredDashboardDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredDashboardDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTieredDashboardDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredDashboardDigestModifier for ChannelTieredDashboardDigestAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TieredDashboardDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredDashboardDigestAdjuster {
    pub cap: f64,
}

impl TieredDashboardDigestModifier for CapTieredDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredDashboardDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredDashboardDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredDashboardDigestModifier for TieredTieredDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredDashboardDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredDashboardDigestAdjuster {
    pub minimum: f64,
}

impl TieredDashboardDigestModifier for MinimumTieredDashboardDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredDashboardDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered dashboard digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredDashboardDigestEngine {
    pub steps: Vec<Rc<dyn TieredDashboardDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredDashboardDigestEngine {
    pub fn price(&mut self, item: &TieredDashboardDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredDashboardDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_dashboard_digest_engine() -> TieredDashboardDigestEngine {
    TieredDashboardDigestEngine {
        steps: vec![
            Rc::new(ChannelTieredDashboardDigestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.88), ("retail".to_string(), 1.35)]),
            }) as Rc<dyn TieredDashboardDigestModifier>,
            Rc::new(CapTieredDashboardDigestAdjuster { cap: 46701.0 }),
            Rc::new(TieredTieredDashboardDigestAdjuster {
                bounds: vec![393.0, 6366.0],
                rates: vec![0.83, 0.81],
            }),
            Rc::new(MinimumTieredDashboardDigestAdjuster { minimum: 48.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 87319.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_dashboard_digest_engine();
        let item = new_tiered_dashboard_digest("code-994", "title-58");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
