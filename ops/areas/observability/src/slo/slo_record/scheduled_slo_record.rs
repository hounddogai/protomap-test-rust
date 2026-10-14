use super::*;

use acme_kit::text;

/// Where a scheduled slo record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSloRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledSloRecordStatus {
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

impl std::fmt::Display for ScheduledSloRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled slo record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecord {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub external_ref: String,
    pub issuer: String,
    pub rate: f64,
    pub ends_at: i64,
    pub status: ScheduledSloRecordStatus,
}

impl ScheduledSloRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSloRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSloRecordStatus) -> bool {
        match self.status {
            ScheduledSloRecordStatus::Draft => {
                return next == ScheduledSloRecordStatus::Pending;
            }
            ScheduledSloRecordStatus::Pending => {
                match next {
                    ScheduledSloRecordStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSloRecordStatus::Active => {
                match next {
                    ScheduledSloRecordStatus::Suspended | ScheduledSloRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSloRecordStatus::Suspended => {
                return next == ScheduledSloRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSloRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled slo record {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        (self.weight * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSloRecord {
        ScheduledSloRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            rate: self.rate,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled slo record in status draft.
pub fn new_scheduled_slo_record(code: &str, name: &str) -> ScheduledSloRecord {
    ScheduledSloRecord {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        external_ref: "".to_string(),
        issuer: "".to_string(),
        rate: 0.0,
        ends_at: 0,
        status: ScheduledSloRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_slo_record("code-684", "name-490");
        assert_eq!(item.status, ScheduledSloRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_slo_record("code-684", "name-490");
        if item.move_to(ScheduledSloRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSloRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_slo_record("code-684", "name-490");
        item.status = ScheduledSloRecordStatus::Closed;
        assert!(
            item.move_to(ScheduledSloRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
