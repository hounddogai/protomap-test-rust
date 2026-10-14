use super::*;

use acme_kit::text;

/// Where a rolling egress manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEgressManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingEgressManifestStatus {
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

impl std::fmt::Display for RollingEgressManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling egress manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifest {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub checksum: String,
    pub country: String,
    pub port: i64,
    pub attempts: i64,
    pub status: RollingEgressManifestStatus,
}

impl RollingEgressManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEgressManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEgressManifestStatus) -> bool {
        match self.status {
            RollingEgressManifestStatus::Planned => {
                return next == RollingEgressManifestStatus::Scheduled
                    || next == RollingEgressManifestStatus::Cancelled;
            }
            RollingEgressManifestStatus::Scheduled => {
                match next {
                    RollingEgressManifestStatus::InProgress | RollingEgressManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressManifestStatus::InProgress => {
                match next {
                    RollingEgressManifestStatus::Completed | RollingEgressManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressManifestStatus::Completed => {
                match next {
                    RollingEgressManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingEgressManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling egress manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling egress manifest", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.port as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEgressManifest {
        RollingEgressManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            checksum: self.checksum.clone(),
            country: self.country.clone(),
            port: self.port,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling egress manifest in status planned.
pub fn new_rolling_egress_manifest(code: &str, title: &str) -> RollingEgressManifest {
    RollingEgressManifest {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        checksum: "".to_string(),
        country: "".to_string(),
        port: 0,
        attempts: 0,
        status: RollingEgressManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_egress_manifest("code-55", "title-412");
        assert_eq!(item.status, RollingEgressManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_egress_manifest("code-55", "title-412");
        if item.move_to(RollingEgressManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingEgressManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_egress_manifest("code-55", "title-412");
        item.status = RollingEgressManifestStatus::Cancelled;
        assert!(
            item.move_to(RollingEgressManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
