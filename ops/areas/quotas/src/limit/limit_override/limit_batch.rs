use super::*;

use acme_kit::text;

/// Where a limit batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LimitBatchStatus {
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

impl std::fmt::Display for LimitBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatch {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub threshold: f64,
    pub country: String,
    pub status: LimitBatchStatus,
}

impl LimitBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitBatchStatus) -> bool {
        match self.status {
            LimitBatchStatus::New => {
                match next {
                    LimitBatchStatus::Queued | LimitBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitBatchStatus::Queued => {
                match next {
                    LimitBatchStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitBatchStatus::Running => {
                match next {
                    LimitBatchStatus::Done | LimitBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitBatchStatus::Done => {
                return next == LimitBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit batch {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "limit batch", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.threshold + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitBatch {
        LimitBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            threshold: self.threshold,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit batch in status new.
pub fn new_limit_batch(code: &str, title: &str) -> LimitBatch {
    LimitBatch {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        threshold: 0.0,
        country: "".to_string(),
        status: LimitBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_limit_batch("code-399", "title-365");
        assert_eq!(item.status, LimitBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_limit_batch("code-399", "title-365");
        if item.move_to(LimitBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_limit_batch("code-399", "title-365");
        item.status = LimitBatchStatus::Failed;
        assert!(item.move_to(LimitBatchStatus::New).is_err(), "expected move to fail");
    }
}
