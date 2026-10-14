use super::*;

use acme_kit::text;

/// Where a silence tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SilenceTierStatus {
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

impl std::fmt::Display for SilenceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence tier as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTier {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub floor_value: f64,
    pub unit: String,
    pub audience: String,
    pub key_id: String,
    pub discount: f64,
    pub external_ref: String,
    pub pinned: bool,
    pub checksum: String,
    pub status: SilenceTierStatus,
}

impl SilenceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceTierStatus) -> bool {
        match self.status {
            SilenceTierStatus::Idle => {
                return next == SilenceTierStatus::Draining;
            }
            SilenceTierStatus::Warming => {
                return next == SilenceTierStatus::Ready || next == SilenceTierStatus::Stopped;
            }
            SilenceTierStatus::Ready => {
                match next {
                    SilenceTierStatus::Draining | SilenceTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceTierStatus::Draining => {
                match next {
                    SilenceTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SilenceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.floor_value - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceTier {
        SilenceTier {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            floor_value: self.floor_value,
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            key_id: self.key_id.clone(),
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a silence tier in status idle.
pub fn new_silence_tier(code: &str, name: &str) -> SilenceTier {
    SilenceTier {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        floor_value: 0.0,
        unit: "".to_string(),
        audience: "".to_string(),
        key_id: "".to_string(),
        discount: 0.0,
        external_ref: "".to_string(),
        pinned: false,
        checksum: "".to_string(),
        status: SilenceTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_silence_tier("code-190", "name-260");
        assert_eq!(item.status, SilenceTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_silence_tier("code-190", "name-260");
        if item.move_to(SilenceTierStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceTierStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_silence_tier("code-190", "name-260");
        item.status = SilenceTierStatus::Stopped;
        assert!(item.move_to(SilenceTierStatus::Idle).is_err(), "expected move to fail");
    }
}
