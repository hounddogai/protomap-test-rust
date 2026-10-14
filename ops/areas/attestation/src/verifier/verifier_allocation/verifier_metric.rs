use super::*;

use acme_kit::text;

/// Where a verifier metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierMetricStatus {
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

impl std::fmt::Display for VerifierMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetric {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub floor_value: f64,
    pub updated_at: i64,
    pub burst: i64,
    pub version: i64,
    pub fingerprint: String,
    pub category: String,
    pub issuer: String,
    pub region: String,
    pub status: VerifierMetricStatus,
}

impl VerifierMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierMetricStatus) -> bool {
        match self.status {
            VerifierMetricStatus::Planned => {
                match next {
                    VerifierMetricStatus::InProgress | VerifierMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierMetricStatus::Scheduled => {
                return next == VerifierMetricStatus::InProgress || next == VerifierMetricStatus::Completed;
            }
            VerifierMetricStatus::InProgress => {
                return next == VerifierMetricStatus::Completed || next == VerifierMetricStatus::Cancelled;
            }
            VerifierMetricStatus::Completed => {
                return next == VerifierMetricStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier metric", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.floor_value * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierMetric {
        VerifierMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            floor_value: self.floor_value,
            updated_at: self.updated_at,
            burst: self.burst,
            version: self.version,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a verifier metric in status planned.
pub fn new_verifier_metric(code: &str, name: &str) -> VerifierMetric {
    VerifierMetric {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        floor_value: 0.0,
        updated_at: 0,
        burst: 0,
        version: 0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        issuer: "".to_string(),
        region: "".to_string(),
        status: VerifierMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_metric("code-157", "name-113");
        assert_eq!(item.status, VerifierMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_verifier_metric("code-157", "name-113");
        if item.move_to(VerifierMetricStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierMetricStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_metric("code-157", "name-113");
        item.status = VerifierMetricStatus::Cancelled;
        assert!(
            item.move_to(VerifierMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
