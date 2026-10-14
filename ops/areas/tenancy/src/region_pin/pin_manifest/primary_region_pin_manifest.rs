use super::*;

use acme_kit::text;

/// Where a primary region pin manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRegionPinManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryRegionPinManifestStatus {
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

impl std::fmt::Display for PrimaryRegionPinManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary region pin manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifest {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub title: String,
    pub audience: String,
    pub fingerprint: String,
    pub status: PrimaryRegionPinManifestStatus,
}

impl PrimaryRegionPinManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRegionPinManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRegionPinManifestStatus) -> bool {
        match self.status {
            PrimaryRegionPinManifestStatus::Idle => {
                return next == PrimaryRegionPinManifestStatus::Ready;
            }
            PrimaryRegionPinManifestStatus::Warming => {
                match next {
                    PrimaryRegionPinManifestStatus::Draining | PrimaryRegionPinManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRegionPinManifestStatus::Ready => {
                match next {
                    PrimaryRegionPinManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRegionPinManifestStatus::Draining => {
                match next {
                    PrimaryRegionPinManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryRegionPinManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary region pin manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary region pin manifest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> PrimaryRegionPinManifest {
        PrimaryRegionPinManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            title: self.title.clone(),
            audience: self.audience.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary region pin manifest in status idle.
pub fn new_primary_region_pin_manifest(code: &str, name: &str) -> PrimaryRegionPinManifest {
    PrimaryRegionPinManifest {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        title: "".to_string(),
        audience: "".to_string(),
        fingerprint: "".to_string(),
        status: PrimaryRegionPinManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_region_pin_manifest("code-70", "name-363");
        assert_eq!(item.status, PrimaryRegionPinManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_region_pin_manifest("code-70", "name-363");
        if item.move_to(PrimaryRegionPinManifestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRegionPinManifestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_region_pin_manifest("code-70", "name-363");
        item.status = PrimaryRegionPinManifestStatus::Stopped;
        assert!(
            item.move_to(PrimaryRegionPinManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
