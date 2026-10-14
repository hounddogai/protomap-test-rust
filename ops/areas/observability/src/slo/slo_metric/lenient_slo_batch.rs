use super::*;

use acme_kit::text;

/// Where a lenient slo batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientSloBatchStatus {
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

impl std::fmt::Display for LenientSloBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient slo batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloBatch {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub limit: i64,
    pub visible: bool,
    pub quota_units: i64,
    pub status: LenientSloBatchStatus,
}

impl LenientSloBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloBatchStatus) -> bool {
        match self.status {
            LenientSloBatchStatus::Planned => {
                return next == LenientSloBatchStatus::InProgress || next == LenientSloBatchStatus::Completed;
            }
            LenientSloBatchStatus::Scheduled => {
                match next {
                    LenientSloBatchStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloBatchStatus::InProgress => {
                match next {
                    LenientSloBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloBatchStatus::Completed => {
                match next {
                    LenientSloBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientSloBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo batch {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.limit as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSloBatch {
        LenientSloBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            limit: self.limit,
            visible: self.visible,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient slo batch in status planned.
pub fn new_lenient_slo_batch(code: &str, name: &str) -> LenientSloBatch {
    LenientSloBatch {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        limit: 0,
        visible: false,
        quota_units: 0,
        status: LenientSloBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_slo_batch("code-62", "name-967");
        assert_eq!(item.status, LenientSloBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_lenient_slo_batch("code-62", "name-967");
        if item.move_to(LenientSloBatchStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloBatchStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_slo_batch("code-62", "name-967");
        item.status = LenientSloBatchStatus::Cancelled;
        assert!(
            item.move_to(LenientSloBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
