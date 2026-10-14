use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped federation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFederationTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedFederationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CappedFederationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped federation target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTarget {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub checksum: String,
    pub source: String,
    pub namespace: String,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub margin: f64,
    pub approved: bool,
    pub status: CappedFederationTargetStatus,
}

impl CappedFederationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFederationTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFederationTargetStatus) -> bool {
        match self.status {
            CappedFederationTargetStatus::Idle => {
                return next == CappedFederationTargetStatus::Ready || next == CappedFederationTargetStatus::Stopped;
            }
            CappedFederationTargetStatus::Warming => {
                match next {
                    CappedFederationTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFederationTargetStatus::Ready => {
                match next {
                    CappedFederationTargetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFederationTargetStatus::Draining => {
                match next {
                    CappedFederationTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedFederationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped federation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped federation target", self.code)
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
        ((self.margin + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedFederationTarget {
        CappedFederationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            margin: self.margin,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped federation target in status idle.
pub fn new_capped_federation_target(code: &str, name: &str) -> CappedFederationTarget {
    CappedFederationTarget {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        checksum: "".to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        attributes: BTreeMap::new(),
        country: "".to_string(),
        margin: 0.0,
        approved: false,
        status: CappedFederationTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_federation_target("code-702", "name-954");
        assert_eq!(item.status, CappedFederationTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_capped_federation_target("code-702", "name-954");
        if item.move_to(CappedFederationTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFederationTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_federation_target("code-702", "name-954");
        item.status = CappedFederationTargetStatus::Stopped;
        assert!(
            item.move_to(CappedFederationTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
