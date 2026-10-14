use super::*;

use acme_kit::text;

/// Where a primary health check variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryHealthCheckVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryHealthCheckVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryHealthCheckVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary health check variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVariance {
    pub code: String,
    pub title: String,
    pub region: String,
    pub approved: bool,
    pub sequence: i64,
    pub locale: String,
    pub duration_minutes: i64,
    pub status: PrimaryHealthCheckVarianceStatus,
}

impl PrimaryHealthCheckVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryHealthCheckVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryHealthCheckVarianceStatus) -> bool {
        match self.status {
            PrimaryHealthCheckVarianceStatus::Proposed => {
                return next == PrimaryHealthCheckVarianceStatus::Approved;
            }
            PrimaryHealthCheckVarianceStatus::Approved => {
                match next {
                    PrimaryHealthCheckVarianceStatus::Rejected | PrimaryHealthCheckVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryHealthCheckVarianceStatus::Rejected => {
                return next == PrimaryHealthCheckVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryHealthCheckVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary health check variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryHealthCheckVariance {
        PrimaryHealthCheckVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            approved: self.approved,
            sequence: self.sequence,
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary health check variance in status proposed.
pub fn new_primary_health_check_variance(code: &str, title: &str) -> PrimaryHealthCheckVariance {
    PrimaryHealthCheckVariance {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        approved: false,
        sequence: 0,
        locale: "".to_string(),
        duration_minutes: 0,
        status: PrimaryHealthCheckVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_health_check_variance("code-224", "title-565");
        assert_eq!(item.status, PrimaryHealthCheckVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_health_check_variance("code-224", "title-565");
        if item.move_to(PrimaryHealthCheckVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryHealthCheckVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_health_check_variance("code-224", "title-565");
        item.status = PrimaryHealthCheckVarianceStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryHealthCheckVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
