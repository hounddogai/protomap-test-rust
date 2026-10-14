use super::*;

use acme_kit::text;

/// Where a tiered timeline notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTimelineNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredTimelineNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredTimelineNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered timeline notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineNotice {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub threshold: f64,
    pub owner_id: String,
    pub weight: f64,
    pub source: String,
    pub status: TieredTimelineNoticeStatus,
}

impl TieredTimelineNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTimelineNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTimelineNoticeStatus) -> bool {
        match self.status {
            TieredTimelineNoticeStatus::Planned => {
                match next {
                    TieredTimelineNoticeStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTimelineNoticeStatus::Scheduled => {
                match next {
                    TieredTimelineNoticeStatus::Completed | TieredTimelineNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTimelineNoticeStatus::InProgress => {
                return next == TieredTimelineNoticeStatus::Completed || next == TieredTimelineNoticeStatus::Cancelled;
            }
            TieredTimelineNoticeStatus::Completed => {
                return next == TieredTimelineNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTimelineNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered timeline notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.threshold * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTimelineNotice {
        TieredTimelineNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            threshold: self.threshold,
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered timeline notice in status planned.
pub fn new_tiered_timeline_notice(code: &str, label: &str) -> TieredTimelineNotice {
    TieredTimelineNotice {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        threshold: 0.0,
        owner_id: "".to_string(),
        weight: 0.0,
        source: "".to_string(),
        status: TieredTimelineNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_timeline_notice("code-269", "label-813");
        assert_eq!(item.status, TieredTimelineNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_timeline_notice("code-269", "label-813");
        if item.move_to(TieredTimelineNoticeStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTimelineNoticeStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_timeline_notice("code-269", "label-813");
        item.status = TieredTimelineNoticeStatus::Cancelled;
        assert!(
            item.move_to(TieredTimelineNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
