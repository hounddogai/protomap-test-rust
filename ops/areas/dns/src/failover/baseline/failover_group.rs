use super::*;

use acme_kit::text;

/// Where a failover group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FailoverGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FailoverGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverGroup {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub limit: i64,
    pub window_days: i64,
    pub source: String,
    pub status: FailoverGroupStatus,
}

impl FailoverGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverGroupStatus) -> bool {
        match self.status {
            FailoverGroupStatus::Requested => {
                match next {
                    FailoverGroupStatus::Granted | FailoverGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverGroupStatus::Granted => {
                return next == FailoverGroupStatus::Expired;
            }
            FailoverGroupStatus::Revoked => {
                return next == FailoverGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover group {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover group", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverGroup {
        FailoverGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            limit: self.limit,
            window_days: self.window_days,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover group in status requested.
pub fn new_failover_group(code: &str, label: &str) -> FailoverGroup {
    FailoverGroup {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        limit: 0,
        window_days: 0,
        source: "".to_string(),
        status: FailoverGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_failover_group("code-532", "label-536");
        assert_eq!(item.status, FailoverGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_failover_group("code-532", "label-536");
        if item.move_to(FailoverGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_failover_group("code-532", "label-536");
        item.status = FailoverGroupStatus::Expired;
        assert!(
            item.move_to(FailoverGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
