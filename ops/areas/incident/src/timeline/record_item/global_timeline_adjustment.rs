use super::*;

use acme_kit::text;

/// Where a global timeline adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTimelineAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalTimelineAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalTimelineAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global timeline adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustment {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub namespace: String,
    pub unit: String,
    pub notes: String,
    pub locale: String,
    pub checksum: String,
    pub ttl_seconds: i64,
    pub status: GlobalTimelineAdjustmentStatus,
}

impl GlobalTimelineAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTimelineAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTimelineAdjustmentStatus) -> bool {
        match self.status {
            GlobalTimelineAdjustmentStatus::Draft => {
                match next {
                    GlobalTimelineAdjustmentStatus::Pending | GlobalTimelineAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTimelineAdjustmentStatus::Pending => {
                match next {
                    GlobalTimelineAdjustmentStatus::Active | GlobalTimelineAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTimelineAdjustmentStatus::Active => {
                return next == GlobalTimelineAdjustmentStatus::Suspended
                    || next == GlobalTimelineAdjustmentStatus::Closed;
            }
            GlobalTimelineAdjustmentStatus::Suspended => {
                match next {
                    GlobalTimelineAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalTimelineAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global timeline adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global timeline adjustment", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTimelineAdjustment {
        GlobalTimelineAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            notes: self.notes.clone(),
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global timeline adjustment in status draft.
pub fn new_global_timeline_adjustment(code: &str, title: &str) -> GlobalTimelineAdjustment {
    GlobalTimelineAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        namespace: "".to_string(),
        unit: "".to_string(),
        notes: "".to_string(),
        locale: "".to_string(),
        checksum: "".to_string(),
        ttl_seconds: 0,
        status: GlobalTimelineAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_timeline_adjustment("code-812", "title-680");
        assert_eq!(item.status, GlobalTimelineAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_timeline_adjustment("code-812", "title-680");
        if item.move_to(GlobalTimelineAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTimelineAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_timeline_adjustment("code-812", "title-680");
        item.status = GlobalTimelineAdjustmentStatus::Closed;
        assert!(
            item.move_to(GlobalTimelineAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
