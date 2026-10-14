use super::*;

use acme_kit::text;

/// Where a isolation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IsolationBaselineStatus {
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

impl std::fmt::Display for IsolationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBaseline {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub maximum: f64,
    pub weight: f64,
    pub status: IsolationBaselineStatus,
}

impl IsolationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationBaselineStatus) -> bool {
        match self.status {
            IsolationBaselineStatus::New => {
                match next {
                    IsolationBaselineStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationBaselineStatus::Queued => {
                return next == IsolationBaselineStatus::Done;
            }
            IsolationBaselineStatus::Running => {
                return next == IsolationBaselineStatus::Done;
            }
            IsolationBaselineStatus::Done => {
                return next == IsolationBaselineStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.weight * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationBaseline {
        IsolationBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a isolation baseline in status new.
pub fn new_isolation_baseline(code: &str, label: &str) -> IsolationBaseline {
    IsolationBaseline {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        maximum: 0.0,
        weight: 0.0,
        status: IsolationBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_isolation_baseline("code-494", "label-303");
        assert_eq!(item.status, IsolationBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_isolation_baseline("code-494", "label-303");
        if item.move_to(IsolationBaselineStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationBaselineStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_isolation_baseline("code-494", "label-303");
        item.status = IsolationBaselineStatus::Failed;
        assert!(
            item.move_to(IsolationBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
