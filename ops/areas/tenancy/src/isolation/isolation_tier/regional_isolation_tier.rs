use super::*;

use acme_kit::text;

/// Where a regional isolation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalIsolationTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalIsolationTierStatus {
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

impl std::fmt::Display for RegionalIsolationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional isolation tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationTier {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub cipher: String,
    pub rate: f64,
    pub external_ref: String,
    pub status: RegionalIsolationTierStatus,
}

impl RegionalIsolationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalIsolationTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalIsolationTierStatus) -> bool {
        match self.status {
            RegionalIsolationTierStatus::Idle => {
                return next == RegionalIsolationTierStatus::Ready || next == RegionalIsolationTierStatus::Stopped;
            }
            RegionalIsolationTierStatus::Warming => {
                match next {
                    RegionalIsolationTierStatus::Draining | RegionalIsolationTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalIsolationTierStatus::Ready => {
                match next {
                    RegionalIsolationTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalIsolationTierStatus::Draining => {
                return next == RegionalIsolationTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalIsolationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional isolation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> RegionalIsolationTier {
        RegionalIsolationTier {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            cipher: self.cipher.clone(),
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional isolation tier in status idle.
pub fn new_regional_isolation_tier(code: &str, label: &str) -> RegionalIsolationTier {
    RegionalIsolationTier {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        cipher: "".to_string(),
        rate: 0.0,
        external_ref: "".to_string(),
        status: RegionalIsolationTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_isolation_tier("code-804", "label-342");
        assert_eq!(item.status, RegionalIsolationTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_isolation_tier("code-804", "label-342");
        if item.move_to(RegionalIsolationTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalIsolationTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_isolation_tier("code-804", "label-342");
        item.status = RegionalIsolationTierStatus::Stopped;
        assert!(
            item.move_to(RegionalIsolationTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
