use super::*;

use acme_kit::text;

/// Where a identity segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentitySegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IdentitySegmentStatus {
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

impl std::fmt::Display for IdentitySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity segment as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySegment {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub unit: String,
    pub phone: String,
    pub window_days: i64,
    pub pinned: bool,
    pub audience: String,
    pub priority: i64,
    pub floor_value: f64,
    pub status: IdentitySegmentStatus,
}

impl IdentitySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentitySegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentitySegmentStatus) -> bool {
        match self.status {
            IdentitySegmentStatus::Idle => {
                match next {
                    IdentitySegmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentitySegmentStatus::Warming => {
                match next {
                    IdentitySegmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentitySegmentStatus::Ready => {
                match next {
                    IdentitySegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentitySegmentStatus::Draining => {
                return next == IdentitySegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentitySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity segment {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentitySegment {
        IdentitySegment {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            window_days: self.window_days,
            pinned: self.pinned,
            audience: self.audience.clone(),
            priority: self.priority,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a identity segment in status idle.
pub fn new_identity_segment(code: &str, name: &str) -> IdentitySegment {
    IdentitySegment {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        unit: "".to_string(),
        phone: "".to_string(),
        window_days: 0,
        pinned: false,
        audience: "".to_string(),
        priority: 0,
        floor_value: 0.0,
        status: IdentitySegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_identity_segment("code-643", "name-225");
        assert_eq!(item.status, IdentitySegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_identity_segment("code-643", "name-225");
        if item.move_to(IdentitySegmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentitySegmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_identity_segment("code-643", "name-225");
        item.status = IdentitySegmentStatus::Stopped;
        assert!(
            item.move_to(IdentitySegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
