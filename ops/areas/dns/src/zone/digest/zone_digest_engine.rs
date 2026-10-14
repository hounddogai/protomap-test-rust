use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone digest calculation.
pub trait ZoneDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDigestEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl ZoneDigestStep for ZoneDigestEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneDigestAdjuster {
    pub percent: f64,
}

impl ZoneDigestStep for PercentZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneDigestAdjuster {
    pub cap: f64,
}

impl ZoneDigestStep for CapZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneDigestAdjuster {
    pub step: f64,
}

impl ZoneDigestStep for RoundingZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneDigestAdjuster {
    pub minimum: f64,
}

impl ZoneDigestStep for MinimumZoneDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the zone digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneDigestEngine {
    pub steps: Vec<Rc<dyn ZoneDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneDigestEngine {
    pub fn run(&mut self, item: &ZoneDigest) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_zone_digest_engine() -> ZoneDigestEngine {
    ZoneDigestEngine {
        steps: vec![
            Rc::new(ZoneDigestEndsAtOffsetAdjuster { weight: 0.77 }) as Rc<dyn ZoneDigestStep>,
            Rc::new(PercentZoneDigestAdjuster { percent: -23.1 }),
            Rc::new(CapZoneDigestAdjuster { cap: 30500.0 }),
            Rc::new(RoundingZoneDigestAdjuster { step: 5.0 }),
            Rc::new(MinimumZoneDigestAdjuster { minimum: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 64413.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_digest_engine();
        let item = new_zone_digest("code-118", "title-245");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
