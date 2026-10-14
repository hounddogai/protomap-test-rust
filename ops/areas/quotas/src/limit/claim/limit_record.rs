use super::*;

use acme_kit::text;

/// Where a limit record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitRecordStatus {
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

impl std::fmt::Display for LimitRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRecord {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub approved: bool,
    pub capacity: i64,
    pub country: String,
    pub reviewer: String,
    pub ends_at: i64,
    pub pinned: bool,
    pub currency: String,
    pub channel: String,
    pub status: LimitRecordStatus,
}

impl LimitRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitRecordStatus) -> bool {
        match self.status {
            LimitRecordStatus::Draft => {
                return next == LimitRecordStatus::Active;
            }
            LimitRecordStatus::Pending => {
                match next {
                    LimitRecordStatus::Active | LimitRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRecordStatus::Active => {
                return next == LimitRecordStatus::Closed;
            }
            LimitRecordStatus::Suspended => {
                match next {
                    LimitRecordStatus::Closed => {
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

    pub fn move_to(&mut self, next: LimitRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit record {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitRecord {
        LimitRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            capacity: self.capacity,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            ends_at: self.ends_at,
            pinned: self.pinned,
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit record in status draft.
pub fn new_limit_record(code: &str, title: &str) -> LimitRecord {
    LimitRecord {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        approved: false,
        capacity: 0,
        country: "".to_string(),
        reviewer: "".to_string(),
        ends_at: 0,
        pinned: false,
        currency: "".to_string(),
        channel: "".to_string(),
        status: LimitRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_limit_record("code-559", "title-868");
        assert_eq!(item.status, LimitRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_limit_record("code-559", "title-868");
        if item.move_to(LimitRecordStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitRecordStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_limit_record("code-559", "title-868");
        item.status = LimitRecordStatus::Closed;
        assert!(item.move_to(LimitRecordStatus::Draft).is_err(), "expected move to fail");
    }
}
