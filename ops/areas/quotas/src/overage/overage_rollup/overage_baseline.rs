use super::*;

use acme_kit::text;

/// Where a overage baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageBaselineStatus {
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

impl std::fmt::Display for OverageBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBaseline {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub active: bool,
    pub name: String,
    pub cost: f64,
    pub status: OverageBaselineStatus,
}

impl OverageBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageBaselineStatus) -> bool {
        match self.status {
            OverageBaselineStatus::New => {
                return next == OverageBaselineStatus::Running || next == OverageBaselineStatus::Done;
            }
            OverageBaselineStatus::Queued => {
                return next == OverageBaselineStatus::Running || next == OverageBaselineStatus::Failed;
            }
            OverageBaselineStatus::Running => {
                return next == OverageBaselineStatus::Done;
            }
            OverageBaselineStatus::Done => {
                match next {
                    OverageBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: OverageBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "overage baseline", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.cost + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageBaseline {
        OverageBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            active: self.active,
            name: self.name.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a overage baseline in status new.
pub fn new_overage_baseline(code: &str, title: &str) -> OverageBaseline {
    OverageBaseline {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        active: false,
        name: "".to_string(),
        cost: 0.0,
        status: OverageBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_baseline("code-513", "title-337");
        assert_eq!(item.status, OverageBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_overage_baseline("code-513", "title-337");
        if item.move_to(OverageBaselineStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageBaselineStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_baseline("code-513", "title-337");
        item.status = OverageBaselineStatus::Failed;
        assert!(
            item.move_to(OverageBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
