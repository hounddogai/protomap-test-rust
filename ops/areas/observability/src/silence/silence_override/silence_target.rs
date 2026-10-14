use super::*;

use acme_kit::text;

/// Where a silence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SilenceTargetStatus {
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

impl std::fmt::Display for SilenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence target as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTarget {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub active: bool,
    pub batch_size: i64,
    pub label: String,
    pub tenant_id: String,
    pub floor_value: f64,
    pub window_days: i64,
    pub segment: String,
    pub status: SilenceTargetStatus,
}

impl SilenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceTargetStatus) -> bool {
        match self.status {
            SilenceTargetStatus::Idle => {
                match next {
                    SilenceTargetStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceTargetStatus::Warming => {
                return next == SilenceTargetStatus::Stopped;
            }
            SilenceTargetStatus::Ready => {
                return next == SilenceTargetStatus::Draining || next == SilenceTargetStatus::Stopped;
            }
            SilenceTargetStatus::Draining => {
                return next == SilenceTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.floor_value - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceTarget {
        SilenceTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            active: self.active,
            batch_size: self.batch_size,
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            floor_value: self.floor_value,
            window_days: self.window_days,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a silence target in status idle.
pub fn new_silence_target(code: &str, title: &str) -> SilenceTarget {
    SilenceTarget {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        active: false,
        batch_size: 0,
        label: "".to_string(),
        tenant_id: "".to_string(),
        floor_value: 0.0,
        window_days: 0,
        segment: "".to_string(),
        status: SilenceTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_silence_target("code-894", "title-597");
        assert_eq!(item.status, SilenceTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_silence_target("code-894", "title-597");
        if item.move_to(SilenceTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_silence_target("code-894", "title-597");
        item.status = SilenceTargetStatus::Stopped;
        assert!(
            item.move_to(SilenceTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
