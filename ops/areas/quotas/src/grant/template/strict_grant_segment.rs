use super::*;

use acme_kit::text;

/// Where a strict grant segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictGrantSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictGrantSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictGrantSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict grant segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantSegment {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub category: String,
    pub rotation_days: i64,
    pub starts_at: i64,
    pub weight: f64,
    pub status: StrictGrantSegmentStatus,
}

impl StrictGrantSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictGrantSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictGrantSegmentStatus) -> bool {
        match self.status {
            StrictGrantSegmentStatus::Requested => {
                match next {
                    StrictGrantSegmentStatus::Granted | StrictGrantSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictGrantSegmentStatus::Granted => {
                match next {
                    StrictGrantSegmentStatus::Revoked | StrictGrantSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictGrantSegmentStatus::Revoked => {
                match next {
                    StrictGrantSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: StrictGrantSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict grant segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict grant segment", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.starts_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictGrantSegment {
        StrictGrantSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            starts_at: self.starts_at,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict grant segment in status requested.
pub fn new_strict_grant_segment(code: &str, title: &str) -> StrictGrantSegment {
    StrictGrantSegment {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        category: "".to_string(),
        rotation_days: 0,
        starts_at: 0,
        weight: 0.0,
        status: StrictGrantSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_grant_segment("code-183", "title-707");
        assert_eq!(item.status, StrictGrantSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_strict_grant_segment("code-183", "title-707");
        if item.move_to(StrictGrantSegmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictGrantSegmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_grant_segment("code-183", "title-707");
        item.status = StrictGrantSegmentStatus::Expired;
        assert!(
            item.move_to(StrictGrantSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
