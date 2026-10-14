use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive postmortem digest calculation.
pub trait AdaptivePostmortemDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAdaptivePostmortemDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptivePostmortemDigestModifier for LabelAdaptivePostmortemDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptivePostmortemDigestAdjuster {
    pub minimum: f64,
}

impl AdaptivePostmortemDigestModifier for MinimumAdaptivePostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptivePostmortemDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptivePostmortemDigestModifier for FlaggedAdaptivePostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptivePostmortemDigestAdjuster {
    pub cap: f64,
}

impl AdaptivePostmortemDigestModifier for CapAdaptivePostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptivePostmortemDigestAdjuster {
    pub percent: f64,
}

impl AdaptivePostmortemDigestModifier for PercentAdaptivePostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl AdaptivePostmortemDigestModifier for AdaptivePostmortemDigestDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptivePostmortemDigest, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the adaptive postmortem digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePostmortemDigestEngine {
    pub steps: Vec<Rc<dyn AdaptivePostmortemDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePostmortemDigestEngine {
    pub fn run(&mut self, item: &AdaptivePostmortemDigest) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptivePostmortemDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePostmortemDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_postmortem_digest_engine(flags: &flags::Registry) -> AdaptivePostmortemDigestEngine {
    AdaptivePostmortemDigestEngine {
        steps: vec![
            Rc::new(LabelAdaptivePostmortemDigestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.97),
                    ("central".to_string(), 1.24),
                    ("west".to_string(), 1.36),
                ]),
            }) as Rc<dyn AdaptivePostmortemDigestModifier>,
            Rc::new(MinimumAdaptivePostmortemDigestAdjuster { minimum: 43.6 }),
            Rc::new(FlaggedAdaptivePostmortemDigestAdjuster {
                flags: flags.clone(),
                flag: "adaptive-postmortem-digest-tiered".to_string(),
                percent: 0.5,
            }),
            Rc::new(CapAdaptivePostmortemDigestAdjuster { cap: 39842.0 }),
            Rc::new(PercentAdaptivePostmortemDigestAdjuster { percent: -1.6 }),
            Rc::new(AdaptivePostmortemDigestDurationMinutesOffsetAdjuster { weight: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 88714.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_postmortem_digest_engine(&flags::new_registry());
        let item = new_adaptive_postmortem_digest("code-433", "label-285");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
