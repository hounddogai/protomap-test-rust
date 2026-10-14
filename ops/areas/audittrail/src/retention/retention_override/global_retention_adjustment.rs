use super::*;

use acme_kit::text;

/// Where a global retention adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRetentionAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalRetentionAdjustmentStatus {
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

impl std::fmt::Display for GlobalRetentionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global retention adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustment {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub owner_id: String,
    pub active: bool,
    pub timezone: String,
    pub archived: bool,
    pub window_days: i64,
    pub status: GlobalRetentionAdjustmentStatus,
}

impl GlobalRetentionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRetentionAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRetentionAdjustmentStatus) -> bool {
        match self.status {
            GlobalRetentionAdjustmentStatus::Idle => {
                match next {
                    GlobalRetentionAdjustmentStatus::Ready | GlobalRetentionAdjustmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionAdjustmentStatus::Warming => {
                match next {
                    GlobalRetentionAdjustmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionAdjustmentStatus::Ready => {
                return next == GlobalRetentionAdjustmentStatus::Draining
                    || next == GlobalRetentionAdjustmentStatus::Stopped;
            }
            GlobalRetentionAdjustmentStatus::Draining => {
                return next == GlobalRetentionAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRetentionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global retention adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.window_days as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRetentionAdjustment {
        GlobalRetentionAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            tags: self.tags.clone(),
            owner_id: self.owner_id.clone(),
            active: self.active,
            timezone: self.timezone.clone(),
            archived: self.archived,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global retention adjustment in status idle.
pub fn new_global_retention_adjustment(code: &str, label: &str) -> GlobalRetentionAdjustment {
    GlobalRetentionAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        tags: Vec::new(),
        owner_id: "".to_string(),
        active: false,
        timezone: "".to_string(),
        archived: false,
        window_days: 0,
        status: GlobalRetentionAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_retention_adjustment("code-938", "label-504");
        assert_eq!(item.status, GlobalRetentionAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_global_retention_adjustment("code-938", "label-504");
        if item.move_to(GlobalRetentionAdjustmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRetentionAdjustmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_retention_adjustment("code-938", "label-504");
        item.status = GlobalRetentionAdjustmentStatus::Stopped;
        assert!(
            item.move_to(GlobalRetentionAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
