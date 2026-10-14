use super::*;

use acme_kit::text;

/// Where a rollout slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSlot {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub ends_at: i64,
    pub cipher: String,
    pub description: String,
    pub segment: String,
    pub discount: f64,
    pub score: f64,
    pub rate: f64,
    pub status: RolloutSlotStatus,
}

impl RolloutSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutSlotStatus) -> bool {
        match self.status {
            RolloutSlotStatus::Requested => {
                return next == RolloutSlotStatus::Granted || next == RolloutSlotStatus::Revoked;
            }
            RolloutSlotStatus::Granted => {
                match next {
                    RolloutSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutSlotStatus::Revoked => {
                match next {
                    RolloutSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: RolloutSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout slot", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutSlot {
        RolloutSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            cipher: self.cipher.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            discount: self.discount,
            score: self.score,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout slot in status requested.
pub fn new_rollout_slot(code: &str, title: &str) -> RolloutSlot {
    RolloutSlot {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        ends_at: 0,
        cipher: "".to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        discount: 0.0,
        score: 0.0,
        rate: 0.0,
        status: RolloutSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_slot("code-692", "title-598");
        assert_eq!(item.status, RolloutSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollout_slot("code-692", "title-598");
        if item.move_to(RolloutSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_slot("code-692", "title-598");
        item.status = RolloutSlotStatus::Expired;
        assert!(
            item.move_to(RolloutSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
