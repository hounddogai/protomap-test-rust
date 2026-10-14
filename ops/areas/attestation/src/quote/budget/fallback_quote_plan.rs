use super::*;

use acme_kit::text;

/// Where a fallback quote plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQuotePlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackQuotePlanStatus {
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

impl std::fmt::Display for FallbackQuotePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback quote plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuotePlan {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub weight: f64,
    pub approved: bool,
    pub pinned: bool,
    pub version: i64,
    pub attempts: i64,
    pub batch_size: i64,
    pub status: FallbackQuotePlanStatus,
}

impl FallbackQuotePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQuotePlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQuotePlanStatus) -> bool {
        match self.status {
            FallbackQuotePlanStatus::Planned => {
                match next {
                    FallbackQuotePlanStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQuotePlanStatus::Scheduled => {
                match next {
                    FallbackQuotePlanStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQuotePlanStatus::InProgress => {
                return next == FallbackQuotePlanStatus::Completed || next == FallbackQuotePlanStatus::Cancelled;
            }
            FallbackQuotePlanStatus::Completed => {
                match next {
                    FallbackQuotePlanStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FallbackQuotePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback quote plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackQuotePlan {
        FallbackQuotePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            weight: self.weight,
            approved: self.approved,
            pinned: self.pinned,
            version: self.version,
            attempts: self.attempts,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback quote plan in status planned.
pub fn new_fallback_quote_plan(code: &str, title: &str) -> FallbackQuotePlan {
    FallbackQuotePlan {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        weight: 0.0,
        approved: false,
        pinned: false,
        version: 0,
        attempts: 0,
        batch_size: 0,
        status: FallbackQuotePlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_quote_plan("code-91", "title-752");
        assert_eq!(item.status, FallbackQuotePlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_fallback_quote_plan("code-91", "title-752");
        if item.move_to(FallbackQuotePlanStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQuotePlanStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_quote_plan("code-91", "title-752");
        item.status = FallbackQuotePlanStatus::Cancelled;
        assert!(
            item.move_to(FallbackQuotePlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
