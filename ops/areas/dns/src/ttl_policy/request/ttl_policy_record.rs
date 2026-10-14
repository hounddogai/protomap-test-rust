use super::*;

use acme_kit::text;

/// Where a ttl policy record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TtlPolicyRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TtlPolicyRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRecord {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub channel: String,
    pub window_days: i64,
    pub fingerprint: String,
    pub country: String,
    pub maximum: f64,
    pub sequence: i64,
    pub burst: i64,
    pub status: TtlPolicyRecordStatus,
}

impl TtlPolicyRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyRecordStatus) -> bool {
        match self.status {
            TtlPolicyRecordStatus::Proposed => {
                match next {
                    TtlPolicyRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRecordStatus::Approved => {
                match next {
                    TtlPolicyRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRecordStatus::Rejected => {
                return next == TtlPolicyRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy record {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy record", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyRecord {
        TtlPolicyRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            channel: self.channel.clone(),
            window_days: self.window_days,
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            sequence: self.sequence,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a ttl policy record in status proposed.
pub fn new_ttl_policy_record(code: &str, label: &str) -> TtlPolicyRecord {
    TtlPolicyRecord {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        channel: "".to_string(),
        window_days: 0,
        fingerprint: "".to_string(),
        country: "".to_string(),
        maximum: 0.0,
        sequence: 0,
        burst: 0,
        status: TtlPolicyRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_ttl_policy_record("code-86", "label-975");
        assert_eq!(item.status, TtlPolicyRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_ttl_policy_record("code-86", "label-975");
        if item.move_to(TtlPolicyRecordStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyRecordStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_ttl_policy_record("code-86", "label-975");
        item.status = TtlPolicyRecordStatus::Withdrawn;
        assert!(
            item.move_to(TtlPolicyRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
