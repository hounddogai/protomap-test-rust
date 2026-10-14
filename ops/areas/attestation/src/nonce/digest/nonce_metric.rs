use super::*;

use acme_kit::text;

/// Where a nonce metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl NonceMetricStatus {
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

impl std::fmt::Display for NonceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetric {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub principal: String,
    pub limit: i64,
    pub score: f64,
    pub status: NonceMetricStatus,
}

impl NonceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceMetricStatus) -> bool {
        match self.status {
            NonceMetricStatus::Open => {
                return next == NonceMetricStatus::Archived;
            }
            NonceMetricStatus::InReview => {
                return next == NonceMetricStatus::Reopened;
            }
            NonceMetricStatus::Resolved => {
                match next {
                    NonceMetricStatus::Reopened | NonceMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceMetricStatus::Reopened => {
                return next == NonceMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceMetric {
        NonceMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            limit: self.limit,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce metric in status open.
pub fn new_nonce_metric(code: &str, title: &str) -> NonceMetric {
    NonceMetric {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        limit: 0,
        score: 0.0,
        status: NonceMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_nonce_metric("code-549", "title-653");
        assert_eq!(item.status, NonceMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_nonce_metric("code-549", "title-653");
        if item.move_to(NonceMetricStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceMetricStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_nonce_metric("code-549", "title-653");
        item.status = NonceMetricStatus::Archived;
        assert!(item.move_to(NonceMetricStatus::Open).is_err(), "expected move to fail");
    }
}
