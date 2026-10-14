use super::*;

use acme_kit::text;

/// Where a adaptive isolation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIsolationGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveIsolationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveIsolationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive isolation group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationGroup {
    pub code: String,
    pub name: String,
    pub title: String,
    pub label: String,
    pub limit: i64,
    pub status: AdaptiveIsolationGroupStatus,
}

impl AdaptiveIsolationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIsolationGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIsolationGroupStatus) -> bool {
        match self.status {
            AdaptiveIsolationGroupStatus::Requested => {
                match next {
                    AdaptiveIsolationGroupStatus::Granted | AdaptiveIsolationGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIsolationGroupStatus::Granted => {
                return next == AdaptiveIsolationGroupStatus::Revoked || next == AdaptiveIsolationGroupStatus::Expired;
            }
            AdaptiveIsolationGroupStatus::Revoked => {
                match next {
                    AdaptiveIsolationGroupStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveIsolationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive isolation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> AdaptiveIsolationGroup {
        AdaptiveIsolationGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive isolation group in status requested.
pub fn new_adaptive_isolation_group(code: &str, name: &str) -> AdaptiveIsolationGroup {
    AdaptiveIsolationGroup {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        label: "".to_string(),
        limit: 0,
        status: AdaptiveIsolationGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_isolation_group("code-781", "name-293");
        assert_eq!(item.status, AdaptiveIsolationGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_adaptive_isolation_group("code-781", "name-293");
        if item.move_to(AdaptiveIsolationGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveIsolationGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_isolation_group("code-781", "name-293");
        item.status = AdaptiveIsolationGroupStatus::Expired;
        assert!(
            item.move_to(AdaptiveIsolationGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
