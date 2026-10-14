use super::*;

use acme_kit::text;

/// Where a measurement score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MeasurementScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for MeasurementScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementScore {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub owner_id: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub status: MeasurementScoreStatus,
}

impl MeasurementScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementScoreStatus) -> bool {
        match self.status {
            MeasurementScoreStatus::Open => {
                match next {
                    MeasurementScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementScoreStatus::InReview => {
                match next {
                    MeasurementScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementScoreStatus::Resolved => {
                return next == MeasurementScoreStatus::Archived;
            }
            MeasurementScoreStatus::Reopened => {
                return next == MeasurementScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementScore {
        MeasurementScore {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a measurement score in status open.
pub fn new_measurement_score(code: &str, title: &str) -> MeasurementScore {
    MeasurementScore {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        owner_id: "".to_string(),
        port: 0,
        ttl_seconds: 0,
        batch_size: 0,
        status: MeasurementScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_measurement_score("code-368", "title-431");
        assert_eq!(item.status, MeasurementScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_measurement_score("code-368", "title-431");
        if item.move_to(MeasurementScoreStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementScoreStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_measurement_score("code-368", "title-431");
        item.status = MeasurementScoreStatus::Archived;
        assert!(
            item.move_to(MeasurementScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
