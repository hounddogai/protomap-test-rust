use super::*;

use acme_kit::text;

/// Where a weighted runbook quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRunbookQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedRunbookQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedRunbookQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted runbook quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookQuota {
    pub code: String,
    pub name: String,
    pub host: String,
    pub scope: String,
    pub segment: String,
    pub maximum: f64,
    pub window_days: i64,
    pub status: WeightedRunbookQuotaStatus,
}

impl WeightedRunbookQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRunbookQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRunbookQuotaStatus) -> bool {
        match self.status {
            WeightedRunbookQuotaStatus::Requested => {
                match next {
                    WeightedRunbookQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRunbookQuotaStatus::Granted => {
                match next {
                    WeightedRunbookQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRunbookQuotaStatus::Revoked => {
                match next {
                    WeightedRunbookQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedRunbookQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted runbook quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.window_days as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRunbookQuota {
        WeightedRunbookQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            scope: self.scope.clone(),
            segment: self.segment.clone(),
            maximum: self.maximum,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted runbook quota in status requested.
pub fn new_weighted_runbook_quota(code: &str, name: &str) -> WeightedRunbookQuota {
    WeightedRunbookQuota {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        scope: "".to_string(),
        segment: "".to_string(),
        maximum: 0.0,
        window_days: 0,
        status: WeightedRunbookQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_runbook_quota("code-937", "name-359");
        assert_eq!(item.status, WeightedRunbookQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_weighted_runbook_quota("code-937", "name-359");
        if item.move_to(WeightedRunbookQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRunbookQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_runbook_quota("code-937", "name-359");
        item.status = WeightedRunbookQuotaStatus::Expired;
        assert!(
            item.move_to(WeightedRunbookQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
