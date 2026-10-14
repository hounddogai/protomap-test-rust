use super::*;

use acme_kit::text;

/// Where a seasonal postmortem manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPostmortemManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalPostmortemManifestStatus {
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

impl std::fmt::Display for SeasonalPostmortemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal postmortem manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemManifest {
    pub code: String,
    pub name: String,
    pub host: String,
    pub summary: String,
    pub audience: String,
    pub window_days: i64,
    pub status: SeasonalPostmortemManifestStatus,
}

impl SeasonalPostmortemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPostmortemManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPostmortemManifestStatus) -> bool {
        match self.status {
            SeasonalPostmortemManifestStatus::Idle => {
                match next {
                    SeasonalPostmortemManifestStatus::Warming | SeasonalPostmortemManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPostmortemManifestStatus::Warming => {
                return next == SeasonalPostmortemManifestStatus::Draining;
            }
            SeasonalPostmortemManifestStatus::Ready => {
                match next {
                    SeasonalPostmortemManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPostmortemManifestStatus::Draining => {
                match next {
                    SeasonalPostmortemManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalPostmortemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal postmortem manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal postmortem manifest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> SeasonalPostmortemManifest {
        SeasonalPostmortemManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal postmortem manifest in status idle.
pub fn new_seasonal_postmortem_manifest(code: &str, name: &str) -> SeasonalPostmortemManifest {
    SeasonalPostmortemManifest {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        summary: "".to_string(),
        audience: "".to_string(),
        window_days: 0,
        status: SeasonalPostmortemManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_postmortem_manifest("code-284", "name-637");
        assert_eq!(item.status, SeasonalPostmortemManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_seasonal_postmortem_manifest("code-284", "name-637");
        if item.move_to(SeasonalPostmortemManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPostmortemManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_postmortem_manifest("code-284", "name-637");
        item.status = SeasonalPostmortemManifestStatus::Stopped;
        assert!(
            item.move_to(SeasonalPostmortemManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
