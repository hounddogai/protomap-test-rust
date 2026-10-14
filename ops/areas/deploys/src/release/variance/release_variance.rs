use super::*;

use acme_kit::text;

/// Where a release variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseVarianceStatus {
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

impl std::fmt::Display for ReleaseVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseVariance {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub version: i64,
    pub source: String,
    pub status: ReleaseVarianceStatus,
}

impl ReleaseVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseVarianceStatus) -> bool {
        match self.status {
            ReleaseVarianceStatus::Open => {
                return next == ReleaseVarianceStatus::InReview || next == ReleaseVarianceStatus::Reopened;
            }
            ReleaseVarianceStatus::InReview => {
                return next == ReleaseVarianceStatus::Archived;
            }
            ReleaseVarianceStatus::Resolved => {
                match next {
                    ReleaseVarianceStatus::Reopened | ReleaseVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseVarianceStatus::Reopened => {
                return next == ReleaseVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.version as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseVariance {
        ReleaseVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            version: self.version,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a release variance in status open.
pub fn new_release_variance(code: &str, label: &str) -> ReleaseVariance {
    ReleaseVariance {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        version: 0,
        source: "".to_string(),
        status: ReleaseVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_variance("code-908", "label-155");
        assert_eq!(item.status, ReleaseVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_release_variance("code-908", "label-155");
        if item.move_to(ReleaseVarianceStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseVarianceStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_variance("code-908", "label-155");
        item.status = ReleaseVarianceStatus::Archived;
        assert!(
            item.move_to(ReleaseVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
