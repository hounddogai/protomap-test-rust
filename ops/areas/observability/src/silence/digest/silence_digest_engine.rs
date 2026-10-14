use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the silence digest calculation.
pub trait SilenceDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceDigestAdjuster {
    pub minimum: f64,
}

impl SilenceDigestAdjuster for MinimumSilenceDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSilenceDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceDigestAdjuster for ScopeSilenceDigestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SilenceDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceDigestAdjuster for FlaggedSilenceDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceDigestEngine {
    pub steps: Vec<Rc<dyn SilenceDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceDigestEngine {
    pub fn run(&mut self, item: &SilenceDigest) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_digest_engine(flags: &flags::Registry) -> SilenceDigestEngine {
    SilenceDigestEngine {
        steps: vec![
            Rc::new(MinimumSilenceDigestAdjuster { minimum: 48.7 }) as Rc<dyn SilenceDigestAdjuster>,
            Rc::new(ScopeSilenceDigestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.28), ("south".to_string(), 0.9)]),
            }),
            Rc::new(FlaggedSilenceDigestAdjuster {
                flags: flags.clone(),
                flag: "silence-digest-capped".to_string(),
                percent: 3.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 46285.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_digest_engine(&flags::new_registry());
        let item = new_silence_digest("code-390", "title-489");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
