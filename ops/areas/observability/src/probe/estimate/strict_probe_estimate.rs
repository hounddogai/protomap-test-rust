use super::*;

use acme_kit::text;

/// Where a strict probe estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictProbeEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictProbeEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictProbeEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict probe estimate as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictProbeEstimate {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub host: String,
    pub minimum: f64,
    pub score: f64,
    pub status: StrictProbeEstimateStatus,
}

impl StrictProbeEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictProbeEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictProbeEstimateStatus) -> bool {
        match self.status {
            StrictProbeEstimateStatus::Requested => {
                match next {
                    StrictProbeEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictProbeEstimateStatus::Granted => {
                match next {
                    StrictProbeEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictProbeEstimateStatus::Revoked => {
                return next == StrictProbeEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictProbeEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict probe estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        (self.minimum * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictProbeEstimate {
        StrictProbeEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            host: self.host.clone(),
            minimum: self.minimum,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict probe estimate in status requested.
pub fn new_strict_probe_estimate(code: &str, title: &str) -> StrictProbeEstimate {
    StrictProbeEstimate {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        host: "".to_string(),
        minimum: 0.0,
        score: 0.0,
        status: StrictProbeEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_probe_estimate("code-17", "title-424");
        assert_eq!(item.status, StrictProbeEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_strict_probe_estimate("code-17", "title-424");
        if item.move_to(StrictProbeEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictProbeEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_probe_estimate("code-17", "title-424");
        item.status = StrictProbeEstimateStatus::Expired;
        assert!(
            item.move_to(StrictProbeEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
