use super::*;

use acme_kit::text;

/// Where a adaptive timeline exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTimelineExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveTimelineExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveTimelineExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive timeline exception as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelineException {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub visible: bool,
    pub retries: i64,
    pub tenant_id: String,
    pub window_days: i64,
    pub label: String,
    pub limit: i64,
    pub ratio: f64,
    pub reviewer: String,
    pub status: AdaptiveTimelineExceptionStatus,
}

impl AdaptiveTimelineException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTimelineExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTimelineExceptionStatus) -> bool {
        match self.status {
            AdaptiveTimelineExceptionStatus::Proposed => {
                match next {
                    AdaptiveTimelineExceptionStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTimelineExceptionStatus::Approved => {
                return next == AdaptiveTimelineExceptionStatus::Rejected;
            }
            AdaptiveTimelineExceptionStatus::Rejected => {
                match next {
                    AdaptiveTimelineExceptionStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveTimelineExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive timeline exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive timeline exception", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTimelineException {
        AdaptiveTimelineException {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            visible: self.visible,
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            label: self.label.clone(),
            limit: self.limit,
            ratio: self.ratio,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive timeline exception in status proposed.
pub fn new_adaptive_timeline_exception(code: &str, title: &str) -> AdaptiveTimelineException {
    AdaptiveTimelineException {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        visible: false,
        retries: 0,
        tenant_id: "".to_string(),
        window_days: 0,
        label: "".to_string(),
        limit: 0,
        ratio: 0.0,
        reviewer: "".to_string(),
        status: AdaptiveTimelineExceptionStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_timeline_exception("code-324", "title-216");
        assert_eq!(item.status, AdaptiveTimelineExceptionStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_timeline_exception("code-324", "title-216");
        if item.move_to(AdaptiveTimelineExceptionStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTimelineExceptionStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_timeline_exception("code-324", "title-216");
        item.status = AdaptiveTimelineExceptionStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveTimelineExceptionStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
