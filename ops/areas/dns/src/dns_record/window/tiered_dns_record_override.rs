use super::*;

use acme_kit::text;

/// Where a tiered dns record override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDnsRecordOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredDnsRecordOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredDnsRecordOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered dns record override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordOverride {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub amount: f64,
    pub currency: String,
    pub segment: String,
    pub status: TieredDnsRecordOverrideStatus,
}

impl TieredDnsRecordOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDnsRecordOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDnsRecordOverrideStatus) -> bool {
        match self.status {
            TieredDnsRecordOverrideStatus::Planned => {
                match next {
                    TieredDnsRecordOverrideStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDnsRecordOverrideStatus::Scheduled => {
                return next == TieredDnsRecordOverrideStatus::Completed;
            }
            TieredDnsRecordOverrideStatus::InProgress => {
                return next == TieredDnsRecordOverrideStatus::Completed;
            }
            TieredDnsRecordOverrideStatus::Completed => {
                match next {
                    TieredDnsRecordOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredDnsRecordOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dns record override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered dns record override", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.amount * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDnsRecordOverride {
        TieredDnsRecordOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            amount: self.amount,
            currency: self.currency.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered dns record override in status planned.
pub fn new_tiered_dns_record_override(code: &str, name: &str) -> TieredDnsRecordOverride {
    TieredDnsRecordOverride {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        amount: 0.0,
        currency: "".to_string(),
        segment: "".to_string(),
        status: TieredDnsRecordOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_dns_record_override("code-6", "name-924");
        assert_eq!(item.status, TieredDnsRecordOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_dns_record_override("code-6", "name-924");
        if item.move_to(TieredDnsRecordOverrideStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDnsRecordOverrideStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_dns_record_override("code-6", "name-924");
        item.status = TieredDnsRecordOverrideStatus::Cancelled;
        assert!(
            item.move_to(TieredDnsRecordOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
