use super::*;

use acme_kit::text;

/// Where a seasonal grant baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalGrantBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalGrantBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SeasonalGrantBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal grant baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantBaseline {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub segment: String,
    pub archived: bool,
    pub scope: String,
    pub notes: String,
    pub status: SeasonalGrantBaselineStatus,
}

impl SeasonalGrantBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalGrantBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalGrantBaselineStatus) -> bool {
        match self.status {
            SeasonalGrantBaselineStatus::Idle => {
                match next {
                    SeasonalGrantBaselineStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantBaselineStatus::Warming => {
                match next {
                    SeasonalGrantBaselineStatus::Ready | SeasonalGrantBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantBaselineStatus::Ready => {
                match next {
                    SeasonalGrantBaselineStatus::Draining | SeasonalGrantBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalGrantBaselineStatus::Draining => {
                return next == SeasonalGrantBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalGrantBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal grant baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> SeasonalGrantBaseline {
        SeasonalGrantBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            segment: self.segment.clone(),
            archived: self.archived,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal grant baseline in status idle.
pub fn new_seasonal_grant_baseline(code: &str, title: &str) -> SeasonalGrantBaseline {
    SeasonalGrantBaseline {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        segment: "".to_string(),
        archived: false,
        scope: "".to_string(),
        notes: "".to_string(),
        status: SeasonalGrantBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_grant_baseline("code-508", "title-494");
        assert_eq!(item.status, SeasonalGrantBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_seasonal_grant_baseline("code-508", "title-494");
        if item.move_to(SeasonalGrantBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalGrantBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_grant_baseline("code-508", "title-494");
        item.status = SeasonalGrantBaselineStatus::Stopped;
        assert!(
            item.move_to(SeasonalGrantBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
