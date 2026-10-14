use super::*;

use acme_kit::text;

/// Where a forecast digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastDigestStatus {
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

impl std::fmt::Display for ForecastDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDigest {
    pub code: String,
    pub name: String,
    pub description: String,
    pub port: i64,
    pub margin: f64,
    pub version: i64,
    pub batch_size: i64,
    pub threshold: f64,
    pub status: ForecastDigestStatus,
}

impl ForecastDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastDigestStatus) -> bool {
        match self.status {
            ForecastDigestStatus::Open => {
                return next == ForecastDigestStatus::Reopened || next == ForecastDigestStatus::Archived;
            }
            ForecastDigestStatus::InReview => {
                return next == ForecastDigestStatus::Resolved || next == ForecastDigestStatus::Archived;
            }
            ForecastDigestStatus::Resolved => {
                return next == ForecastDigestStatus::Archived;
            }
            ForecastDigestStatus::Reopened => {
                match next {
                    ForecastDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: ForecastDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.margin * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastDigest {
        ForecastDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            port: self.port,
            margin: self.margin,
            version: self.version,
            batch_size: self.batch_size,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a forecast digest in status open.
pub fn new_forecast_digest(code: &str, name: &str) -> ForecastDigest {
    ForecastDigest {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        port: 0,
        margin: 0.0,
        version: 0,
        batch_size: 0,
        threshold: 0.0,
        status: ForecastDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_forecast_digest("code-93", "name-614");
        assert_eq!(item.status, ForecastDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_forecast_digest("code-93", "name-614");
        if item.move_to(ForecastDigestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastDigestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_forecast_digest("code-93", "name-614");
        item.status = ForecastDigestStatus::Archived;
        assert!(
            item.move_to(ForecastDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
