use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a federation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FederationProfileStatus {
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

impl std::fmt::Display for FederationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationProfile {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub limit: i64,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub region: String,
    pub scope: String,
    pub status: FederationProfileStatus,
}

impl FederationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationProfileStatus) -> bool {
        match self.status {
            FederationProfileStatus::Planned => {
                match next {
                    FederationProfileStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationProfileStatus::Scheduled => {
                match next {
                    FederationProfileStatus::InProgress | FederationProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationProfileStatus::InProgress => {
                match next {
                    FederationProfileStatus::Completed | FederationProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationProfileStatus::Completed => {
                match next {
                    FederationProfileStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FederationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation profile", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_load(&self) -> f64 {
        (self.ratio * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationProfile {
        FederationProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            limit: self.limit,
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            region: self.region.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a federation profile in status planned.
pub fn new_federation_profile(code: &str, name: &str) -> FederationProfile {
    FederationProfile {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        limit: 0,
        ratio: 0.0,
        duration_minutes: 0,
        burst: 0,
        region: "".to_string(),
        scope: "".to_string(),
        status: FederationProfileStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_federation_profile("code-188", "name-287");
        assert_eq!(item.status, FederationProfileStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_federation_profile("code-188", "name-287");
        if item.move_to(FederationProfileStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationProfileStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_federation_profile("code-188", "name-287");
        item.status = FederationProfileStatus::Cancelled;
        assert!(
            item.move_to(FederationProfileStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
