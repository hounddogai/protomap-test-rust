use super::*;

use acme_kit::text;

/// Where a fallback retention record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRetentionRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackRetentionRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FallbackRetentionRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback retention record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionRecord {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub amount: f64,
    pub retries: i64,
    pub region: String,
    pub grace_days: i64,
    pub source: String,
    pub quota_units: i64,
    pub status: FallbackRetentionRecordStatus,
}

impl FallbackRetentionRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRetentionRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRetentionRecordStatus) -> bool {
        match self.status {
            FallbackRetentionRecordStatus::New => {
                match next {
                    FallbackRetentionRecordStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionRecordStatus::Queued => {
                return next == FallbackRetentionRecordStatus::Running;
            }
            FallbackRetentionRecordStatus::Running => {
                return next == FallbackRetentionRecordStatus::Done || next == FallbackRetentionRecordStatus::Failed;
            }
            FallbackRetentionRecordStatus::Done => {
                match next {
                    FallbackRetentionRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackRetentionRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback retention record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback retention record", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.grace_days as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRetentionRecord {
        FallbackRetentionRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            amount: self.amount,
            retries: self.retries,
            region: self.region.clone(),
            grace_days: self.grace_days,
            source: self.source.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback retention record in status new.
pub fn new_fallback_retention_record(code: &str, label: &str) -> FallbackRetentionRecord {
    FallbackRetentionRecord {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        amount: 0.0,
        retries: 0,
        region: "".to_string(),
        grace_days: 0,
        source: "".to_string(),
        quota_units: 0,
        status: FallbackRetentionRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_retention_record("code-931", "label-640");
        assert_eq!(item.status, FallbackRetentionRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_fallback_retention_record("code-931", "label-640");
        if item.move_to(FallbackRetentionRecordStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRetentionRecordStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_retention_record("code-931", "label-640");
        item.status = FallbackRetentionRecordStatus::Failed;
        assert!(
            item.move_to(FallbackRetentionRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
