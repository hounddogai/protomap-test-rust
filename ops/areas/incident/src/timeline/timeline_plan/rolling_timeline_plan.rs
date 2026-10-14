use super::*;

use acme_kit::text;

/// Where a rolling timeline plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTimelinePlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingTimelinePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingTimelinePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling timeline plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelinePlan {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub updated_at: i64,
    pub phone: String,
    pub threshold: f64,
    pub sequence: i64,
    pub key_id: String,
    pub region: String,
    pub status: RollingTimelinePlanStatus,
}

impl RollingTimelinePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTimelinePlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTimelinePlanStatus) -> bool {
        match self.status {
            RollingTimelinePlanStatus::Requested => {
                return next == RollingTimelinePlanStatus::Granted || next == RollingTimelinePlanStatus::Expired;
            }
            RollingTimelinePlanStatus::Granted => {
                return next == RollingTimelinePlanStatus::Revoked;
            }
            RollingTimelinePlanStatus::Revoked => {
                match next {
                    RollingTimelinePlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingTimelinePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling timeline plan {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTimelinePlan {
        RollingTimelinePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            threshold: self.threshold,
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling timeline plan in status requested.
pub fn new_rolling_timeline_plan(code: &str, name: &str) -> RollingTimelinePlan {
    RollingTimelinePlan {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        updated_at: 0,
        phone: "".to_string(),
        threshold: 0.0,
        sequence: 0,
        key_id: "".to_string(),
        region: "".to_string(),
        status: RollingTimelinePlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_timeline_plan("code-362", "name-127");
        assert_eq!(item.status, RollingTimelinePlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_timeline_plan("code-362", "name-127");
        if item.move_to(RollingTimelinePlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTimelinePlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_timeline_plan("code-362", "name-127");
        item.status = RollingTimelinePlanStatus::Expired;
        assert!(
            item.move_to(RollingTimelinePlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
