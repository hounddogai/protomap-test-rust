use super::*;

use acme_kit::text;

/// Where a capped dashboard review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDashboardReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedDashboardReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedDashboardReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped dashboard review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardReview {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub locale: String,
    pub limit: i64,
    pub duration_minutes: i64,
    pub title: String,
    pub status: CappedDashboardReviewStatus,
}

impl CappedDashboardReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDashboardReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDashboardReviewStatus) -> bool {
        match self.status {
            CappedDashboardReviewStatus::Requested => {
                return next == CappedDashboardReviewStatus::Expired;
            }
            CappedDashboardReviewStatus::Granted => {
                return next == CappedDashboardReviewStatus::Expired;
            }
            CappedDashboardReviewStatus::Revoked => {
                match next {
                    CappedDashboardReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedDashboardReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dashboard review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped dashboard review", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDashboardReview {
        CappedDashboardReview {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            locale: self.locale.clone(),
            limit: self.limit,
            duration_minutes: self.duration_minutes,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped dashboard review in status requested.
pub fn new_capped_dashboard_review(code: &str, label: &str) -> CappedDashboardReview {
    CappedDashboardReview {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        locale: "".to_string(),
        limit: 0,
        duration_minutes: 0,
        title: "".to_string(),
        status: CappedDashboardReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_dashboard_review("code-773", "label-639");
        assert_eq!(item.status, CappedDashboardReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_capped_dashboard_review("code-773", "label-639");
        if item.move_to(CappedDashboardReviewStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDashboardReviewStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_dashboard_review("code-773", "label-639");
        item.status = CappedDashboardReviewStatus::Expired;
        assert!(
            item.move_to(CappedDashboardReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
