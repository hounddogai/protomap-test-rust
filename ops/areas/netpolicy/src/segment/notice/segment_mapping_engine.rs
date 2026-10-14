use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment mapping calculation.
pub trait SegmentMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentMapping, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentMappingFactor for FlaggedSegmentMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentMappingAdjuster {
    pub percent: f64,
}

impl SegmentMappingFactor for PercentSegmentMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentMappingAdjuster {
    pub cap: f64,
}

impl SegmentMappingFactor for CapSegmentMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentMappingAdjuster {
    pub step: f64,
}

impl SegmentMappingFactor for RoundingSegmentMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the segment mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentMappingEngine {
    pub steps: Vec<Rc<dyn SegmentMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentMappingEngine {
    pub fn run(&mut self, item: &SegmentMapping) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_segment_mapping_engine(flags: &flags::Registry) -> SegmentMappingEngine {
    SegmentMappingEngine {
        steps: vec![
            Rc::new(FlaggedSegmentMappingAdjuster {
                flags: flags.clone(),
                flag: "segment-mapping-rolling".to_string(),
                percent: -5.9,
            }) as Rc<dyn SegmentMappingFactor>,
            Rc::new(PercentSegmentMappingAdjuster { percent: -1.0 }),
            Rc::new(CapSegmentMappingAdjuster { cap: 28624.0 }),
            Rc::new(RoundingSegmentMappingAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 30346.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_mapping_engine(&flags::new_registry());
        let item = new_segment_mapping("code-93", "title-156");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
