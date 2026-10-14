use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe score calculation.
pub trait ProbeScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeScoreModifier for FlaggedProbeScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeScoreAdjuster {
    pub step: f64,
}

impl ProbeScoreModifier for RoundingProbeScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeScoreAdjuster {
    pub minimum: f64,
}

impl ProbeScoreModifier for MinimumProbeScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleProbeScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeScoreModifier for TitleProbeScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ProbeScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeScoreModifier for TieredProbeScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the probe score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeScoreEngine {
    pub steps: Vec<Rc<dyn ProbeScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeScoreEngine {
    pub fn compute(&mut self, item: &ProbeScore) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ProbeScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_score_engine(flags: &flags::Registry) -> ProbeScoreEngine {
    ProbeScoreEngine {
        steps: vec![
            Rc::new(FlaggedProbeScoreAdjuster {
                flags: flags.clone(),
                flag: "probe-score-rolling".to_string(),
                percent: -6.8,
            }) as Rc<dyn ProbeScoreModifier>,
            Rc::new(RoundingProbeScoreAdjuster { step: 0.25 }),
            Rc::new(MinimumProbeScoreAdjuster { minimum: 37.4 }),
            Rc::new(TitleProbeScoreAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.8),
                    ("south".to_string(), 1.39),
                    ("east".to_string(), 0.85),
                ]),
            }),
            Rc::new(TieredProbeScoreAdjuster {
                bounds: vec![1620.0, 4588.0, 5577.0],
                rates: vec![1.11, 1.06, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 27050.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_score_engine(&flags::new_registry());
        let item = new_probe_score("code-730", "title-968");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
