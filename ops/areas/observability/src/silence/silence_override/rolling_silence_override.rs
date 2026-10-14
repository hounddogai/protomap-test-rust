use super::*;

use acme_kit::text;

/// Where a rolling silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSilenceOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingSilenceOverrideStatus {
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

impl std::fmt::Display for RollingSilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling silence override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceOverride {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub maximum: f64,
    pub country: String,
    pub archived: bool,
    pub ratio: f64,
    pub status: RollingSilenceOverrideStatus,
}

impl RollingSilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSilenceOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSilenceOverrideStatus) -> bool {
        match self.status {
            RollingSilenceOverrideStatus::Draft => {
                return next == RollingSilenceOverrideStatus::Pending || next == RollingSilenceOverrideStatus::Closed;
            }
            RollingSilenceOverrideStatus::Pending => {
                return next == RollingSilenceOverrideStatus::Active || next == RollingSilenceOverrideStatus::Closed;
            }
            RollingSilenceOverrideStatus::Active => {
                match next {
                    RollingSilenceOverrideStatus::Suspended | RollingSilenceOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSilenceOverrideStatus::Suspended => {
                match next {
                    RollingSilenceOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingSilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling silence override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling silence override", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.maximum + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSilenceOverride {
        RollingSilenceOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            maximum: self.maximum,
            country: self.country.clone(),
            archived: self.archived,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling silence override in status draft.
pub fn new_rolling_silence_override(code: &str, title: &str) -> RollingSilenceOverride {
    RollingSilenceOverride {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        maximum: 0.0,
        country: "".to_string(),
        archived: false,
        ratio: 0.0,
        status: RollingSilenceOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_silence_override("code-995", "title-608");
        assert_eq!(item.status, RollingSilenceOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_silence_override("code-995", "title-608");
        if item.move_to(RollingSilenceOverrideStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSilenceOverrideStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_silence_override("code-995", "title-608");
        item.status = RollingSilenceOverrideStatus::Closed;
        assert!(
            item.move_to(RollingSilenceOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
