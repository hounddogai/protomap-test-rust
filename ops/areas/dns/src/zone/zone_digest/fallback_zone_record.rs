use super::*;

use acme_kit::text;

/// Where a fallback zone record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackZoneRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackZoneRecordStatus {
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

impl std::fmt::Display for FallbackZoneRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback zone record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneRecord {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub quota_units: i64,
    pub score: f64,
    pub status: FallbackZoneRecordStatus,
}

impl FallbackZoneRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackZoneRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackZoneRecordStatus) -> bool {
        match self.status {
            FallbackZoneRecordStatus::New => {
                return next == FallbackZoneRecordStatus::Done;
            }
            FallbackZoneRecordStatus::Queued => {
                return next == FallbackZoneRecordStatus::Failed;
            }
            FallbackZoneRecordStatus::Running => {
                match next {
                    FallbackZoneRecordStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackZoneRecordStatus::Done => {
                return next == FallbackZoneRecordStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackZoneRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback zone record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback zone record", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quota_units as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackZoneRecord {
        FallbackZoneRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback zone record in status new.
pub fn new_fallback_zone_record(code: &str, title: &str) -> FallbackZoneRecord {
    FallbackZoneRecord {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        score: 0.0,
        status: FallbackZoneRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_zone_record("code-553", "title-485");
        assert_eq!(item.status, FallbackZoneRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_fallback_zone_record("code-553", "title-485");
        if item.move_to(FallbackZoneRecordStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackZoneRecordStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_zone_record("code-553", "title-485");
        item.status = FallbackZoneRecordStatus::Failed;
        assert!(
            item.move_to(FallbackZoneRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
