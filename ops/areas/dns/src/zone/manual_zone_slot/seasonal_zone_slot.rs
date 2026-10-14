use super::*;

use acme_kit::text;

/// Where a seasonal zone slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalZoneSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalZoneSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalZoneSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal zone slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlot {
    pub code: String,
    pub name: String,
    pub description: String,
    pub minimum: f64,
    pub created_at: i64,
    pub status: SeasonalZoneSlotStatus,
}

impl SeasonalZoneSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalZoneSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalZoneSlotStatus) -> bool {
        match self.status {
            SeasonalZoneSlotStatus::Requested => {
                return next == SeasonalZoneSlotStatus::Granted || next == SeasonalZoneSlotStatus::Expired;
            }
            SeasonalZoneSlotStatus::Granted => {
                match next {
                    SeasonalZoneSlotStatus::Revoked | SeasonalZoneSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalZoneSlotStatus::Revoked => {
                return next == SeasonalZoneSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalZoneSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal zone slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalZoneSlot {
        SeasonalZoneSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal zone slot in status requested.
pub fn new_seasonal_zone_slot(code: &str, name: &str) -> SeasonalZoneSlot {
    SeasonalZoneSlot {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        minimum: 0.0,
        created_at: 0,
        status: SeasonalZoneSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_zone_slot("code-482", "name-30");
        assert_eq!(item.status, SeasonalZoneSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_zone_slot("code-482", "name-30");
        if item.move_to(SeasonalZoneSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalZoneSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_zone_slot("code-482", "name-30");
        item.status = SeasonalZoneSlotStatus::Expired;
        assert!(
            item.move_to(SeasonalZoneSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
