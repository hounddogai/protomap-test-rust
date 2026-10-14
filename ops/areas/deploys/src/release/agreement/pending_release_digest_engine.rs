use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending release digest calculation.
pub trait PendingReleaseDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingReleaseDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingReleaseDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingReleaseDigestModifier for TitlePendingReleaseDigestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingReleaseDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingReleaseDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingReleaseDigestModifier for FlaggedPendingReleaseDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingReleaseDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingReleaseDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingReleaseDigestModifier for TieredPendingReleaseDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingReleaseDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigestRatioOffsetAdjuster {
    pub weight: f64,
}

impl PendingReleaseDigestModifier for PendingReleaseDigestRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PendingReleaseDigest, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the pending release digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingReleaseDigestEngine {
    pub steps: Vec<Rc<dyn PendingReleaseDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingReleaseDigestEngine {
    pub fn run(&mut self, item: &PendingReleaseDigest) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingReleaseDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_release_digest_engine(flags: &flags::Registry) -> PendingReleaseDigestEngine {
    PendingReleaseDigestEngine {
        steps: vec![
            Rc::new(TitlePendingReleaseDigestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93), ("south".to_string(), 1.12)]),
            }) as Rc<dyn PendingReleaseDigestModifier>,
            Rc::new(FlaggedPendingReleaseDigestAdjuster {
                flags: flags.clone(),
                flag: "pending-release-digest-seasonal".to_string(),
                percent: -9.6,
            }),
            Rc::new(TieredPendingReleaseDigestAdjuster {
                bounds: vec![5964.0, 6236.0, 6543.0],
                rates: vec![1.14, 0.87, 0.94],
            }),
            Rc::new(PendingReleaseDigestRatioOffsetAdjuster { weight: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 43217.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_release_digest_engine(&flags::new_registry());
        let item = new_pending_release_digest("code-159", "title-695");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
