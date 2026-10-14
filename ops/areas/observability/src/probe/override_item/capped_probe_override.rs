use super::*;

use acme_kit::text;

/// Where a capped probe override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedProbeOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedProbeOverrideStatus {
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

impl std::fmt::Display for CappedProbeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped probe override as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverride {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub region: String,
    pub scope: String,
    pub sequence: i64,
    pub owner_id: String,
    pub label: String,
    pub revision: i64,
    pub status: CappedProbeOverrideStatus,
}

impl CappedProbeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedProbeOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedProbeOverrideStatus) -> bool {
        match self.status {
            CappedProbeOverrideStatus::Idle => {
                return next == CappedProbeOverrideStatus::Stopped;
            }
            CappedProbeOverrideStatus::Warming => {
                match next {
                    CappedProbeOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedProbeOverrideStatus::Ready => {
                return next == CappedProbeOverrideStatus::Stopped;
            }
            CappedProbeOverrideStatus::Draining => {
                return next == CappedProbeOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedProbeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped probe override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedProbeOverride {
        CappedProbeOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            region: self.region.clone(),
            scope: self.scope.clone(),
            sequence: self.sequence,
            owner_id: self.owner_id.clone(),
            label: self.label.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped probe override in status idle.
pub fn new_capped_probe_override(code: &str, name: &str) -> CappedProbeOverride {
    CappedProbeOverride {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        region: "".to_string(),
        scope: "".to_string(),
        sequence: 0,
        owner_id: "".to_string(),
        label: "".to_string(),
        revision: 0,
        status: CappedProbeOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_probe_override("code-250", "name-763");
        assert_eq!(item.status, CappedProbeOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_capped_probe_override("code-250", "name-763");
        if item.move_to(CappedProbeOverrideStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedProbeOverrideStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_probe_override("code-250", "name-763");
        item.status = CappedProbeOverrideStatus::Stopped;
        assert!(
            item.move_to(CappedProbeOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
