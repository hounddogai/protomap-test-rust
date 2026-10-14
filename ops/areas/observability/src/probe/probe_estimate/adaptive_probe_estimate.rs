use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive probe estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveProbeEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveProbeEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for AdaptiveProbeEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive probe estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEstimate {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub quota_units: i64,
    pub budget: f64,
    pub burst: i64,
    pub region: String,
    pub attributes: BTreeMap<String, String>,
    pub status: AdaptiveProbeEstimateStatus,
}

impl AdaptiveProbeEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveProbeEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveProbeEstimateStatus) -> bool {
        match self.status {
            AdaptiveProbeEstimateStatus::Planned => {
                return next == AdaptiveProbeEstimateStatus::Completed
                    || next == AdaptiveProbeEstimateStatus::Cancelled;
            }
            AdaptiveProbeEstimateStatus::Scheduled => {
                match next {
                    AdaptiveProbeEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeEstimateStatus::InProgress => {
                match next {
                    AdaptiveProbeEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeEstimateStatus::Completed => {
                match next {
                    AdaptiveProbeEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveProbeEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive probe estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive probe estimate", self.code)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveProbeEstimate {
        AdaptiveProbeEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            quota_units: self.quota_units,
            budget: self.budget,
            burst: self.burst,
            region: self.region.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive probe estimate in status planned.
pub fn new_adaptive_probe_estimate(code: &str, name: &str) -> AdaptiveProbeEstimate {
    AdaptiveProbeEstimate {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        quota_units: 0,
        budget: 0.0,
        burst: 0,
        region: "".to_string(),
        attributes: BTreeMap::new(),
        status: AdaptiveProbeEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_probe_estimate("code-167", "name-632");
        assert_eq!(item.status, AdaptiveProbeEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_adaptive_probe_estimate("code-167", "name-632");
        if item.move_to(AdaptiveProbeEstimateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveProbeEstimateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_probe_estimate("code-167", "name-632");
        item.status = AdaptiveProbeEstimateStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveProbeEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
