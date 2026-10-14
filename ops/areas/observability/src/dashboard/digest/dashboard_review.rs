use super::*;

use acme_kit::text;

/// Where a dashboard review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DashboardReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReview {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub starts_at: i64,
    pub visible: bool,
    pub ttl_seconds: i64,
    pub status: DashboardReviewStatus,
}

impl DashboardReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardReviewStatus) -> bool {
        match self.status {
            DashboardReviewStatus::Idle => {
                return next == DashboardReviewStatus::Warming || next == DashboardReviewStatus::Stopped;
            }
            DashboardReviewStatus::Warming => {
                match next {
                    DashboardReviewStatus::Ready | DashboardReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardReviewStatus::Ready => {
                return next == DashboardReviewStatus::Draining || next == DashboardReviewStatus::Stopped;
            }
            DashboardReviewStatus::Draining => {
                match next {
                    DashboardReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DashboardReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard review {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardReview {
        DashboardReview {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            starts_at: self.starts_at,
            visible: self.visible,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dashboard review in status idle.
pub fn new_dashboard_review(code: &str, name: &str) -> DashboardReview {
    DashboardReview {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        starts_at: 0,
        visible: false,
        ttl_seconds: 0,
        status: DashboardReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dashboard_review("code-162", "name-856");
        assert_eq!(item.status, DashboardReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dashboard_review("code-162", "name-856");
        if item.move_to(DashboardReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dashboard_review("code-162", "name-856");
        item.status = DashboardReviewStatus::Stopped;
        assert!(
            item.move_to(DashboardReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
