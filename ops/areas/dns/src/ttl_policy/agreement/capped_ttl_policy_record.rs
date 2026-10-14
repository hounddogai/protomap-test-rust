use super::*;

use acme_kit::text;

/// Where a capped ttl policy record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTtlPolicyRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedTtlPolicyRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedTtlPolicyRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped ttl policy record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyRecord {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub label: String,
    pub margin: f64,
    pub floor_value: f64,
    pub archived: bool,
    pub phone: String,
    pub status: CappedTtlPolicyRecordStatus,
}

impl CappedTtlPolicyRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTtlPolicyRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTtlPolicyRecordStatus) -> bool {
        match self.status {
            CappedTtlPolicyRecordStatus::Requested => {
                return next == CappedTtlPolicyRecordStatus::Expired;
            }
            CappedTtlPolicyRecordStatus::Granted => {
                match next {
                    CappedTtlPolicyRecordStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTtlPolicyRecordStatus::Revoked => {
                return next == CappedTtlPolicyRecordStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTtlPolicyRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped ttl policy record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped ttl policy record", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTtlPolicyRecord {
        CappedTtlPolicyRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            label: self.label.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            archived: self.archived,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped ttl policy record in status requested.
pub fn new_capped_ttl_policy_record(code: &str, title: &str) -> CappedTtlPolicyRecord {
    CappedTtlPolicyRecord {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        label: "".to_string(),
        margin: 0.0,
        floor_value: 0.0,
        archived: false,
        phone: "".to_string(),
        status: CappedTtlPolicyRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_ttl_policy_record("code-704", "title-716");
        assert_eq!(item.status, CappedTtlPolicyRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_capped_ttl_policy_record("code-704", "title-716");
        if item.move_to(CappedTtlPolicyRecordStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTtlPolicyRecordStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_ttl_policy_record("code-704", "title-716");
        item.status = CappedTtlPolicyRecordStatus::Expired;
        assert!(
            item.move_to(CappedTtlPolicyRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
