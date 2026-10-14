use super::*;

use acme_kit::text;

/// Where a nonce segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl NonceSegmentStatus {
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

impl std::fmt::Display for NonceSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegment {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub rotation_days: i64,
    pub ceiling: f64,
    pub status: NonceSegmentStatus,
}

impl NonceSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceSegmentStatus) -> bool {
        match self.status {
            NonceSegmentStatus::Draft => {
                return next == NonceSegmentStatus::Closed;
            }
            NonceSegmentStatus::Pending => {
                return next == NonceSegmentStatus::Closed;
            }
            NonceSegmentStatus::Active => {
                return next == NonceSegmentStatus::Suspended;
            }
            NonceSegmentStatus::Suspended => {
                return next == NonceSegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        (self.minimum * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceSegment {
        NonceSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            rotation_days: self.rotation_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce segment in status draft.
pub fn new_nonce_segment(code: &str, name: &str) -> NonceSegment {
    NonceSegment {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        rotation_days: 0,
        ceiling: 0.0,
        status: NonceSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_nonce_segment("code-980", "name-986");
        assert_eq!(item.status, NonceSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_nonce_segment("code-980", "name-986");
        if item.move_to(NonceSegmentStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceSegmentStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_nonce_segment("code-980", "name-986");
        item.status = NonceSegmentStatus::Closed;
        assert!(
            item.move_to(NonceSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
