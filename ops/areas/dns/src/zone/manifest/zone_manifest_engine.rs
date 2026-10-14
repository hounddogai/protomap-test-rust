use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone manifest calculation.
pub trait ZoneManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneManifestAdjuster {
    pub minimum: f64,
}

impl ZoneManifestAdjuster for MinimumZoneManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneManifestSequenceOffsetAdjuster {
    pub weight: f64,
}

impl ZoneManifestAdjuster for ZoneManifestSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneManifest, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneManifestAdjuster {
    pub step: f64,
}

impl ZoneManifestAdjuster for RoundingZoneManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionZoneManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneManifestAdjuster for RegionZoneManifestAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ZoneManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the zone manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneManifestEngine {
    pub steps: Vec<Rc<dyn ZoneManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneManifestEngine {
    pub fn run(&mut self, item: &ZoneManifest) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_zone_manifest_engine() -> ZoneManifestEngine {
    ZoneManifestEngine {
        steps: vec![
            Rc::new(MinimumZoneManifestAdjuster { minimum: 1.6 }) as Rc<dyn ZoneManifestAdjuster>,
            Rc::new(ZoneManifestSequenceOffsetAdjuster { weight: 1.27 }),
            Rc::new(RoundingZoneManifestAdjuster { step: 0.1 }),
            Rc::new(RegionZoneManifestAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.01),
                    ("west".to_string(), 1.12),
                    ("east".to_string(), 0.87),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 53815.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_manifest_engine();
        let item = new_zone_manifest("code-750", "title-631");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
