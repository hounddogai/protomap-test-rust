use super::*;

use acme_kit::text;

/// Where a lenient nonce metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNonceMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientNonceMetricStatus {
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

impl std::fmt::Display for LenientNonceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient nonce metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetric {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub discount: f64,
    pub category: String,
    pub status: LenientNonceMetricStatus,
}

impl LenientNonceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNonceMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNonceMetricStatus) -> bool {
        match self.status {
            LenientNonceMetricStatus::Open => {
                return next == LenientNonceMetricStatus::Resolved || next == LenientNonceMetricStatus::Archived;
            }
            LenientNonceMetricStatus::InReview => {
                return next == LenientNonceMetricStatus::Reopened;
            }
            LenientNonceMetricStatus::Resolved => {
                return next == LenientNonceMetricStatus::Reopened || next == LenientNonceMetricStatus::Archived;
            }
            LenientNonceMetricStatus::Reopened => {
                match next {
                    LenientNonceMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientNonceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient nonce metric", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.margin * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientNonceMetric {
        LenientNonceMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            discount: self.discount,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient nonce metric in status open.
pub fn new_lenient_nonce_metric(code: &str, title: &str) -> LenientNonceMetric {
    LenientNonceMetric {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        discount: 0.0,
        category: "".to_string(),
        status: LenientNonceMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_nonce_metric("code-603", "title-100");
        assert_eq!(item.status, LenientNonceMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_nonce_metric("code-603", "title-100");
        if item.move_to(LenientNonceMetricStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientNonceMetricStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_nonce_metric("code-603", "title-100");
        item.status = LenientNonceMetricStatus::Archived;
        assert!(
            item.move_to(LenientNonceMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
