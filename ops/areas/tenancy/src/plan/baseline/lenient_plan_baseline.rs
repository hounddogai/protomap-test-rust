use super::*;

use acme_kit::text;

/// Where a lenient plan baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientPlanBaselineStatus {
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

impl std::fmt::Display for LenientPlanBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient plan baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaseline {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub timezone: String,
    pub visible: bool,
    pub updated_at: i64,
    pub status: LenientPlanBaselineStatus,
}

impl LenientPlanBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanBaselineStatus) -> bool {
        match self.status {
            LenientPlanBaselineStatus::Planned => {
                return next == LenientPlanBaselineStatus::Cancelled;
            }
            LenientPlanBaselineStatus::Scheduled => {
                return next == LenientPlanBaselineStatus::InProgress || next == LenientPlanBaselineStatus::Completed;
            }
            LenientPlanBaselineStatus::InProgress => {
                return next == LenientPlanBaselineStatus::Completed || next == LenientPlanBaselineStatus::Cancelled;
            }
            LenientPlanBaselineStatus::Completed => {
                return next == LenientPlanBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPlanBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.attempts as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPlanBaseline {
        LenientPlanBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            visible: self.visible,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient plan baseline in status planned.
pub fn new_lenient_plan_baseline(code: &str, label: &str) -> LenientPlanBaseline {
    LenientPlanBaseline {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        timezone: "".to_string(),
        visible: false,
        updated_at: 0,
        status: LenientPlanBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_plan_baseline("code-539", "label-917");
        assert_eq!(item.status, LenientPlanBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_lenient_plan_baseline("code-539", "label-917");
        if item.move_to(LenientPlanBaselineStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanBaselineStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_plan_baseline("code-539", "label-917");
        item.status = LenientPlanBaselineStatus::Cancelled;
        assert!(
            item.move_to(LenientPlanBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
