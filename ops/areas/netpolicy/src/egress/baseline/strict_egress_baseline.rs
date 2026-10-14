use super::*;

use acme_kit::text;

/// Where a strict egress baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEgressBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictEgressBaselineStatus {
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

impl std::fmt::Display for StrictEgressBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict egress baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBaseline {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub ratio: f64,
    pub port: i64,
    pub status: StrictEgressBaselineStatus,
}

impl StrictEgressBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEgressBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEgressBaselineStatus) -> bool {
        match self.status {
            StrictEgressBaselineStatus::New => {
                match next {
                    StrictEgressBaselineStatus::Queued | StrictEgressBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressBaselineStatus::Queued => {
                match next {
                    StrictEgressBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressBaselineStatus::Running => {
                return next == StrictEgressBaselineStatus::Failed;
            }
            StrictEgressBaselineStatus::Done => {
                return next == StrictEgressBaselineStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEgressBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict egress baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict egress baseline", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ratio + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEgressBaseline {
        StrictEgressBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            ratio: self.ratio,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict egress baseline in status new.
pub fn new_strict_egress_baseline(code: &str, label: &str) -> StrictEgressBaseline {
    StrictEgressBaseline {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        ratio: 0.0,
        port: 0,
        status: StrictEgressBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_egress_baseline("code-252", "label-160");
        assert_eq!(item.status, StrictEgressBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_egress_baseline("code-252", "label-160");
        if item.move_to(StrictEgressBaselineStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEgressBaselineStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_egress_baseline("code-252", "label-160");
        item.status = StrictEgressBaselineStatus::Failed;
        assert!(
            item.move_to(StrictEgressBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
