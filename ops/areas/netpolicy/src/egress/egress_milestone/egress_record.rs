use super::*;

use acme_kit::text;

/// Where a egress record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressRecordStatus {
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

impl std::fmt::Display for EgressRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecord {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub attempts: i64,
    pub updated_at: i64,
    pub country: String,
    pub scope: String,
    pub channel: String,
    pub active: bool,
    pub status: EgressRecordStatus,
}

impl EgressRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressRecordStatus) -> bool {
        match self.status {
            EgressRecordStatus::Draft => {
                return next == EgressRecordStatus::Pending || next == EgressRecordStatus::Suspended;
            }
            EgressRecordStatus::Pending => {
                match next {
                    EgressRecordStatus::Active | EgressRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRecordStatus::Active => {
                return next == EgressRecordStatus::Closed;
            }
            EgressRecordStatus::Suspended => {
                return next == EgressRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress record {} cannot move from {} to {}",
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
        format!("{} {}", "egress record", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressRecord {
        EgressRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            attempts: self.attempts,
            updated_at: self.updated_at,
            country: self.country.clone(),
            scope: self.scope.clone(),
            channel: self.channel.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a egress record in status draft.
pub fn new_egress_record(code: &str, title: &str) -> EgressRecord {
    EgressRecord {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        attempts: 0,
        updated_at: 0,
        country: "".to_string(),
        scope: "".to_string(),
        channel: "".to_string(),
        active: false,
        status: EgressRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_record("code-502", "title-318");
        assert_eq!(item.status, EgressRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_record("code-502", "title-318");
        if item.move_to(EgressRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_record("code-502", "title-318");
        item.status = EgressRecordStatus::Closed;
        assert!(
            item.move_to(EgressRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
