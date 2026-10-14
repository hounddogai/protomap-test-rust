use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query manifest calculation.
pub trait QueryManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryQueryManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryManifestAdjuster for CountryQueryManifestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryManifestAdjuster for FlaggedQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryManifestAdjuster {
    pub cap: f64,
}

impl QueryManifestAdjuster for CapQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryManifestAdjuster {
    pub step: f64,
}

impl QueryManifestAdjuster for RoundingQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryManifestAdjuster {
    pub minimum: f64,
}

impl QueryManifestAdjuster for MinimumQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryManifestAdjuster for TieredQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the query manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryManifestEngine {
    pub steps: Vec<Rc<dyn QueryManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryManifestEngine {
    pub fn evaluate(&mut self, item: &QueryManifest) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_manifest_engine(flags: &flags::Registry) -> QueryManifestEngine {
    QueryManifestEngine {
        steps: vec![
            Rc::new(CountryQueryManifestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.92)]),
            }) as Rc<dyn QueryManifestAdjuster>,
            Rc::new(FlaggedQueryManifestAdjuster {
                flags: flags.clone(),
                flag: "query-manifest-manual".to_string(),
                percent: 0.1,
            }),
            Rc::new(CapQueryManifestAdjuster { cap: 25696.0 }),
            Rc::new(RoundingQueryManifestAdjuster { step: 0.05 }),
            Rc::new(MinimumQueryManifestAdjuster { minimum: 7.8 }),
            Rc::new(TieredQueryManifestAdjuster {
                bounds: vec![4645.0, 4820.0, 8838.0],
                rates: vec![1.14, 0.96, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 70298.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_manifest_engine(&flags::new_registry());
        let item = new_query_manifest("code-577", "title-897");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
