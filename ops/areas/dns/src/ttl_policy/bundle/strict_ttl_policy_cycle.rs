use super::*;

use acme_kit::text;

/// Where a strict ttl policy cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictTtlPolicyCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictTtlPolicyCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictTtlPolicyCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict ttl policy cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTtlPolicyCycle {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub principal: String,
    pub ends_at: i64,
    pub region: String,
    pub starts_at: i64,
    pub capacity: i64,
    pub cost: f64,
    pub title: String,
    pub weight: f64,
    pub status: StrictTtlPolicyCycleStatus,
}

impl StrictTtlPolicyCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictTtlPolicyCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictTtlPolicyCycleStatus) -> bool {
        match self.status {
            StrictTtlPolicyCycleStatus::Requested => {
                return next == StrictTtlPolicyCycleStatus::Expired;
            }
            StrictTtlPolicyCycleStatus::Granted => {
                return next == StrictTtlPolicyCycleStatus::Revoked || next == StrictTtlPolicyCycleStatus::Expired;
            }
            StrictTtlPolicyCycleStatus::Revoked => {
                match next {
                    StrictTtlPolicyCycleStatus::Expired => {
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

    pub fn move_to(&mut self, next: StrictTtlPolicyCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict ttl policy cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict ttl policy cycle", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictTtlPolicyCycle {
        StrictTtlPolicyCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            principal: self.principal.clone(),
            ends_at: self.ends_at,
            region: self.region.clone(),
            starts_at: self.starts_at,
            capacity: self.capacity,
            cost: self.cost,
            title: self.title.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict ttl policy cycle in status requested.
pub fn new_strict_ttl_policy_cycle(code: &str, name: &str) -> StrictTtlPolicyCycle {
    StrictTtlPolicyCycle {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        principal: "".to_string(),
        ends_at: 0,
        region: "".to_string(),
        starts_at: 0,
        capacity: 0,
        cost: 0.0,
        title: "".to_string(),
        weight: 0.0,
        status: StrictTtlPolicyCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_ttl_policy_cycle("code-441", "name-391");
        assert_eq!(item.status, StrictTtlPolicyCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_strict_ttl_policy_cycle("code-441", "name-391");
        if item.move_to(StrictTtlPolicyCycleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictTtlPolicyCycleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_ttl_policy_cycle("code-441", "name-391");
        item.status = StrictTtlPolicyCycleStatus::Expired;
        assert!(
            item.move_to(StrictTtlPolicyCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
