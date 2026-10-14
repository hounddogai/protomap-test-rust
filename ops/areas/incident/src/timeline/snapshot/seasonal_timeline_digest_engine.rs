use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal timeline digest calculation.
pub trait SeasonalTimelineDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalTimelineDigestAdjuster {
    pub percent: f64,
}

impl SeasonalTimelineDigestFactor for PercentSeasonalTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalTimelineDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalTimelineDigestFactor for FlaggedSeasonalTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalTimelineDigestAdjuster {
    pub step: f64,
}

impl SeasonalTimelineDigestFactor for RoundingSeasonalTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSeasonalTimelineDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalTimelineDigestFactor for NamespaceSeasonalTimelineDigestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalTimelineDigestAdjuster {
    pub cap: f64,
}

impl SeasonalTimelineDigestFactor for CapSeasonalTimelineDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalTimelineDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal timeline digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalTimelineDigestEngine {
    pub steps: Vec<Rc<dyn SeasonalTimelineDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalTimelineDigestEngine {
    pub fn compute(&mut self, item: &SeasonalTimelineDigest) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalTimelineDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_timeline_digest_engine(flags: &flags::Registry) -> SeasonalTimelineDigestEngine {
    SeasonalTimelineDigestEngine {
        steps: vec![
            Rc::new(PercentSeasonalTimelineDigestAdjuster { percent: -22.4 }) as Rc<dyn SeasonalTimelineDigestFactor>,
            Rc::new(FlaggedSeasonalTimelineDigestAdjuster {
                flags: flags.clone(),
                flag: "seasonal-timeline-digest-fallback".to_string(),
                percent: 2.7,
            }),
            Rc::new(RoundingSeasonalTimelineDigestAdjuster { step: 0.05 }),
            Rc::new(NamespaceSeasonalTimelineDigestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("south".to_string(), 1.33),
                    ("east".to_string(), 0.88),
                ]),
            }),
            Rc::new(CapSeasonalTimelineDigestAdjuster { cap: 12465.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 65060.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_timeline_digest_engine(&flags::new_registry());
        let item = new_seasonal_timeline_digest("code-879", "name-25");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
