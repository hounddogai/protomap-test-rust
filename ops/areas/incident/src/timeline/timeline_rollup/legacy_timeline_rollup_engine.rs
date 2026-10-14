use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy timeline rollup calculation.
pub trait LegacyTimelineRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyTimelineRollupAdjuster {
    pub minimum: f64,
}

impl LegacyTimelineRollupModifier for MinimumLegacyTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLegacyTimelineRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyTimelineRollupModifier for CategoryLegacyTimelineRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollupExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyTimelineRollupModifier for LegacyTimelineRollupExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTimelineRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTimelineRollupModifier for FlaggedLegacyTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyTimelineRollupAdjuster {
    pub cap: f64,
}

impl LegacyTimelineRollupModifier for CapLegacyTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyTimelineRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy timeline rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTimelineRollupEngine {
    pub steps: Vec<Rc<dyn LegacyTimelineRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTimelineRollupEngine {
    pub fn compute(&mut self, item: &LegacyTimelineRollup) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTimelineRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_timeline_rollup_engine(flags: &flags::Registry) -> LegacyTimelineRollupEngine {
    LegacyTimelineRollupEngine {
        steps: vec![
            Rc::new(MinimumLegacyTimelineRollupAdjuster { minimum: 27.6 }) as Rc<dyn LegacyTimelineRollupModifier>,
            Rc::new(CategoryLegacyTimelineRollupAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.93),
                    ("east".to_string(), 0.94),
                    ("north".to_string(), 1.38),
                ]),
            }),
            Rc::new(LegacyTimelineRollupExpiresAtOffsetAdjuster { weight: 1.77 }),
            Rc::new(FlaggedLegacyTimelineRollupAdjuster {
                flags: flags.clone(),
                flag: "legacy-timeline-rollup-tiered".to_string(),
                percent: -3.2,
            }),
            Rc::new(CapLegacyTimelineRollupAdjuster { cap: 47516.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 46627.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_timeline_rollup_engine(&flags::new_registry());
        let item = new_legacy_timeline_rollup("code-674", "title-434");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
