use super::*;

use acme_kit::text;

/// Where a fallback alert record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAlertRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackAlertRecordStatus {
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

impl std::fmt::Display for FallbackAlertRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback alert record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRecord {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub cost: f64,
    pub minimum: f64,
    pub source: String,
    pub region: String,
    pub status: FallbackAlertRecordStatus,
}

impl FallbackAlertRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAlertRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAlertRecordStatus) -> bool {
        match self.status {
            FallbackAlertRecordStatus::Draft => {
                return next == FallbackAlertRecordStatus::Pending || next == FallbackAlertRecordStatus::Suspended;
            }
            FallbackAlertRecordStatus::Pending => {
                return next == FallbackAlertRecordStatus::Closed;
            }
            FallbackAlertRecordStatus::Active => {
                match next {
                    FallbackAlertRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAlertRecordStatus::Suspended => {
                return next == FallbackAlertRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackAlertRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback alert record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAlertRecord {
        FallbackAlertRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            cost: self.cost,
            minimum: self.minimum,
            source: self.source.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback alert record in status draft.
pub fn new_fallback_alert_record(code: &str, name: &str) -> FallbackAlertRecord {
    FallbackAlertRecord {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        cost: 0.0,
        minimum: 0.0,
        source: "".to_string(),
        region: "".to_string(),
        status: FallbackAlertRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_alert_record("code-936", "name-662");
        assert_eq!(item.status, FallbackAlertRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_alert_record("code-936", "name-662");
        if item.move_to(FallbackAlertRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackAlertRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_alert_record("code-936", "name-662");
        item.status = FallbackAlertRecordStatus::Closed;
        assert!(
            item.move_to(FallbackAlertRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
