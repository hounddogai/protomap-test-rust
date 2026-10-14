use super::*;

use acme_kit::text;

/// Where a legacy probe group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyProbeGroupStatus {
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

impl std::fmt::Display for LegacyProbeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe group as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroup {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub retries: i64,
    pub principal: String,
    pub segment: String,
    pub source: String,
    pub grace_days: i64,
    pub cipher: String,
    pub phone: String,
    pub status: LegacyProbeGroupStatus,
}

impl LegacyProbeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeGroupStatus) -> bool {
        match self.status {
            LegacyProbeGroupStatus::Idle => {
                return next == LegacyProbeGroupStatus::Warming;
            }
            LegacyProbeGroupStatus::Warming => {
                return next == LegacyProbeGroupStatus::Ready || next == LegacyProbeGroupStatus::Stopped;
            }
            LegacyProbeGroupStatus::Ready => {
                match next {
                    LegacyProbeGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeGroupStatus::Draining => {
                return next == LegacyProbeGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyProbeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeGroup {
        LegacyProbeGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            retries: self.retries,
            principal: self.principal.clone(),
            segment: self.segment.clone(),
            source: self.source.clone(),
            grace_days: self.grace_days,
            cipher: self.cipher.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy probe group in status idle.
pub fn new_legacy_probe_group(code: &str, name: &str) -> LegacyProbeGroup {
    LegacyProbeGroup {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        retries: 0,
        principal: "".to_string(),
        segment: "".to_string(),
        source: "".to_string(),
        grace_days: 0,
        cipher: "".to_string(),
        phone: "".to_string(),
        status: LegacyProbeGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_probe_group("code-704", "name-215");
        assert_eq!(item.status, LegacyProbeGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_probe_group("code-704", "name-215");
        if item.move_to(LegacyProbeGroupStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeGroupStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_probe_group("code-704", "name-215");
        item.status = LegacyProbeGroupStatus::Stopped;
        assert!(
            item.move_to(LegacyProbeGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
