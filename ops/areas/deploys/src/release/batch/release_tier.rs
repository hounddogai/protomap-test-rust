use super::*;

use acme_kit::text;

/// Where a release tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseTierStatus {
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

impl std::fmt::Display for ReleaseTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTier {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub unit: String,
    pub status: ReleaseTierStatus,
}

impl ReleaseTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseTierStatus) -> bool {
        match self.status {
            ReleaseTierStatus::Draft => {
                match next {
                    ReleaseTierStatus::Pending | ReleaseTierStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseTierStatus::Pending => {
                match next {
                    ReleaseTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseTierStatus::Active => {
                return next == ReleaseTierStatus::Suspended;
            }
            ReleaseTierStatus::Suspended => {
                return next == ReleaseTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseTier {
        ReleaseTier {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a release tier in status draft.
pub fn new_release_tier(code: &str, title: &str) -> ReleaseTier {
    ReleaseTier {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        floor_value: 0.0,
        duration_minutes: 0,
        burst: 0,
        unit: "".to_string(),
        status: ReleaseTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_tier("code-45", "title-853");
        assert_eq!(item.status, ReleaseTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_release_tier("code-45", "title-853");
        if item.move_to(ReleaseTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_tier("code-45", "title-853");
        item.status = ReleaseTierStatus::Closed;
        assert!(item.move_to(ReleaseTierStatus::Draft).is_err(), "expected move to fail");
    }
}
