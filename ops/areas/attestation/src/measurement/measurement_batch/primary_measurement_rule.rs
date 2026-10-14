use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary measurement rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryMeasurementRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryMeasurementRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimaryMeasurementRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary measurement rule as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMeasurementRule {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub region: String,
    pub segment: String,
    pub port: i64,
    pub unit: String,
    pub burst: i64,
    pub checksum: String,
    pub status: PrimaryMeasurementRuleStatus,
}

impl PrimaryMeasurementRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryMeasurementRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryMeasurementRuleStatus) -> bool {
        match self.status {
            PrimaryMeasurementRuleStatus::Idle => {
                match next {
                    PrimaryMeasurementRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryMeasurementRuleStatus::Warming => {
                return next == PrimaryMeasurementRuleStatus::Ready || next == PrimaryMeasurementRuleStatus::Stopped;
            }
            PrimaryMeasurementRuleStatus::Ready => {
                match next {
                    PrimaryMeasurementRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryMeasurementRuleStatus::Draining => {
                return next == PrimaryMeasurementRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryMeasurementRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary measurement rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary measurement rule", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.margin * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryMeasurementRule {
        PrimaryMeasurementRule {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            port: self.port,
            unit: self.unit.clone(),
            burst: self.burst,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary measurement rule in status idle.
pub fn new_primary_measurement_rule(code: &str, title: &str) -> PrimaryMeasurementRule {
    PrimaryMeasurementRule {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        region: "".to_string(),
        segment: "".to_string(),
        port: 0,
        unit: "".to_string(),
        burst: 0,
        checksum: "".to_string(),
        status: PrimaryMeasurementRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_measurement_rule("code-958", "title-754");
        assert_eq!(item.status, PrimaryMeasurementRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_primary_measurement_rule("code-958", "title-754");
        if item.move_to(PrimaryMeasurementRuleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryMeasurementRuleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_measurement_rule("code-958", "title-754");
        item.status = PrimaryMeasurementRuleStatus::Stopped;
        assert!(
            item.move_to(PrimaryMeasurementRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
