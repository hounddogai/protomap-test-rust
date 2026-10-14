use super::*;

use acme_kit::text;

/// Where a segment item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentItemStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SegmentItemStatus {
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

impl std::fmt::Display for SegmentItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItem {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub cluster: String,
    pub principal: String,
    pub revision: i64,
    pub window_days: i64,
    pub reviewer: String,
    pub grace_days: i64,
    pub status: SegmentItemStatus,
}

impl SegmentItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentItemStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentItemStatus) -> bool {
        match self.status {
            SegmentItemStatus::Idle => {
                match next {
                    SegmentItemStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentItemStatus::Warming => {
                return next == SegmentItemStatus::Stopped;
            }
            SegmentItemStatus::Ready => {
                return next == SegmentItemStatus::Draining || next == SegmentItemStatus::Stopped;
            }
            SegmentItemStatus::Draining => {
                return next == SegmentItemStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentItem {
        SegmentItem {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            cluster: self.cluster.clone(),
            principal: self.principal.clone(),
            revision: self.revision,
            window_days: self.window_days,
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a segment item in status idle.
pub fn new_segment_item(code: &str, name: &str) -> SegmentItem {
    SegmentItem {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        cluster: "".to_string(),
        principal: "".to_string(),
        revision: 0,
        window_days: 0,
        reviewer: "".to_string(),
        grace_days: 0,
        status: SegmentItemStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_segment_item("code-472", "name-356");
        assert_eq!(item.status, SegmentItemStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_segment_item("code-472", "name-356");
        if item.move_to(SegmentItemStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentItemStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_segment_item("code-472", "name-356");
        item.status = SegmentItemStatus::Stopped;
        assert!(item.move_to(SegmentItemStatus::Idle).is_err(), "expected move to fail");
    }
}
