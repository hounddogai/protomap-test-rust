use super::*;

use acme_kit::text;

/// Where a release window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseWindowStatus {
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

impl std::fmt::Display for ReleaseWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseWindow {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub principal: String,
    pub region: String,
    pub minimum: f64,
    pub rate: f64,
    pub starts_at: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub weight: f64,
    pub status: ReleaseWindowStatus,
}

impl ReleaseWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseWindowStatus) -> bool {
        match self.status {
            ReleaseWindowStatus::Open => {
                match next {
                    ReleaseWindowStatus::Resolved | ReleaseWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseWindowStatus::InReview => {
                return next == ReleaseWindowStatus::Resolved || next == ReleaseWindowStatus::Archived;
            }
            ReleaseWindowStatus::Resolved => {
                return next == ReleaseWindowStatus::Reopened || next == ReleaseWindowStatus::Archived;
            }
            ReleaseWindowStatus::Reopened => {
                return next == ReleaseWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.minimum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseWindow {
        ReleaseWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            rate: self.rate,
            starts_at: self.starts_at,
            burst: self.burst,
            quota_units: self.quota_units,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a release window in status open.
pub fn new_release_window(code: &str, label: &str) -> ReleaseWindow {
    ReleaseWindow {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        principal: "".to_string(),
        region: "".to_string(),
        minimum: 0.0,
        rate: 0.0,
        starts_at: 0,
        burst: 0,
        quota_units: 0,
        weight: 0.0,
        status: ReleaseWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_window("code-314", "label-346");
        assert_eq!(item.status, ReleaseWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_release_window("code-314", "label-346");
        if item.move_to(ReleaseWindowStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseWindowStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_window("code-314", "label-346");
        item.status = ReleaseWindowStatus::Archived;
        assert!(
            item.move_to(ReleaseWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
