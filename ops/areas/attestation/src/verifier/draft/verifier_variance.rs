use super::*;

use acme_kit::text;

/// Where a verifier variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierVarianceStatus {
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

impl std::fmt::Display for VerifierVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierVariance {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub sequence: i64,
    pub minimum: f64,
    pub floor_value: f64,
    pub window_days: i64,
    pub burst: i64,
    pub pinned: bool,
    pub fingerprint: String,
    pub updated_at: i64,
    pub status: VerifierVarianceStatus,
}

impl VerifierVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierVarianceStatus) -> bool {
        match self.status {
            VerifierVarianceStatus::Planned => {
                match next {
                    VerifierVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierVarianceStatus::Scheduled => {
                return next == VerifierVarianceStatus::InProgress;
            }
            VerifierVarianceStatus::InProgress => {
                match next {
                    VerifierVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierVarianceStatus::Completed => {
                match next {
                    VerifierVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: VerifierVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        (self.floor_value * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierVariance {
        VerifierVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            sequence: self.sequence,
            minimum: self.minimum,
            floor_value: self.floor_value,
            window_days: self.window_days,
            burst: self.burst,
            pinned: self.pinned,
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a verifier variance in status planned.
pub fn new_verifier_variance(code: &str, name: &str) -> VerifierVariance {
    VerifierVariance {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        sequence: 0,
        minimum: 0.0,
        floor_value: 0.0,
        window_days: 0,
        burst: 0,
        pinned: false,
        fingerprint: "".to_string(),
        updated_at: 0,
        status: VerifierVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_variance("code-610", "name-834");
        assert_eq!(item.status, VerifierVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_verifier_variance("code-610", "name-834");
        if item.move_to(VerifierVarianceStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierVarianceStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_variance("code-610", "name-834");
        item.status = VerifierVarianceStatus::Cancelled;
        assert!(
            item.move_to(VerifierVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
