use super::*;

use acme_kit::text;

/// Where a lenient suspension threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSuspensionThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientSuspensionThresholdStatus {
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

impl std::fmt::Display for LenientSuspensionThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient suspension threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThreshold {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub grace_days: i64,
    pub budget: f64,
    pub quantity: i64,
    pub minimum: f64,
    pub status: LenientSuspensionThresholdStatus,
}

impl LenientSuspensionThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSuspensionThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSuspensionThresholdStatus) -> bool {
        match self.status {
            LenientSuspensionThresholdStatus::New => {
                match next {
                    LenientSuspensionThresholdStatus::Queued | LenientSuspensionThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionThresholdStatus::Queued => {
                return next == LenientSuspensionThresholdStatus::Failed;
            }
            LenientSuspensionThresholdStatus::Running => {
                return next == LenientSuspensionThresholdStatus::Done
                    || next == LenientSuspensionThresholdStatus::Failed;
            }
            LenientSuspensionThresholdStatus::Done => {
                return next == LenientSuspensionThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSuspensionThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient suspension threshold {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSuspensionThreshold {
        LenientSuspensionThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            grace_days: self.grace_days,
            budget: self.budget,
            quantity: self.quantity,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient suspension threshold in status new.
pub fn new_lenient_suspension_threshold(code: &str, title: &str) -> LenientSuspensionThreshold {
    LenientSuspensionThreshold {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        grace_days: 0,
        budget: 0.0,
        quantity: 0,
        minimum: 0.0,
        status: LenientSuspensionThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_suspension_threshold("code-569", "title-636");
        assert_eq!(item.status, LenientSuspensionThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_suspension_threshold("code-569", "title-636");
        if item.move_to(LenientSuspensionThresholdStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSuspensionThresholdStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_suspension_threshold("code-569", "title-636");
        item.status = LenientSuspensionThresholdStatus::Failed;
        assert!(
            item.move_to(LenientSuspensionThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
