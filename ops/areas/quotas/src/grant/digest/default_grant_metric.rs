use super::*;

use acme_kit::text;

/// Where a default grant metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultGrantMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultGrantMetricStatus {
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

impl std::fmt::Display for DefaultGrantMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default grant metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantMetric {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub principal: String,
    pub cipher: String,
    pub reviewer: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub scope: String,
    pub status: DefaultGrantMetricStatus,
}

impl DefaultGrantMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultGrantMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultGrantMetricStatus) -> bool {
        match self.status {
            DefaultGrantMetricStatus::Open => {
                return next == DefaultGrantMetricStatus::Archived;
            }
            DefaultGrantMetricStatus::InReview => {
                match next {
                    DefaultGrantMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultGrantMetricStatus::Resolved => {
                return next == DefaultGrantMetricStatus::Reopened;
            }
            DefaultGrantMetricStatus::Reopened => {
                return next == DefaultGrantMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultGrantMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default grant metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default grant metric", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultGrantMetric {
        DefaultGrantMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default grant metric in status open.
pub fn new_default_grant_metric(code: &str, title: &str) -> DefaultGrantMetric {
    DefaultGrantMetric {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        principal: "".to_string(),
        cipher: "".to_string(),
        reviewer: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        status: DefaultGrantMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_grant_metric("code-410", "title-555");
        assert_eq!(item.status, DefaultGrantMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_default_grant_metric("code-410", "title-555");
        if item.move_to(DefaultGrantMetricStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultGrantMetricStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_grant_metric("code-410", "title-555");
        item.status = DefaultGrantMetricStatus::Archived;
        assert!(
            item.move_to(DefaultGrantMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
