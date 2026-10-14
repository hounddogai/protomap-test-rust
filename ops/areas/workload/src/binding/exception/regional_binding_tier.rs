use super::*;

use acme_kit::text;

/// Where a regional binding tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBindingTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalBindingTierStatus {
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

impl std::fmt::Display for RegionalBindingTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional binding tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingTier {
    pub code: String,
    pub label: String,
    pub source: String,
    pub timezone: String,
    pub ends_at: i64,
    pub checksum: String,
    pub status: RegionalBindingTierStatus,
}

impl RegionalBindingTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBindingTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBindingTierStatus) -> bool {
        match self.status {
            RegionalBindingTierStatus::Idle => {
                return next == RegionalBindingTierStatus::Draining || next == RegionalBindingTierStatus::Stopped;
            }
            RegionalBindingTierStatus::Warming => {
                match next {
                    RegionalBindingTierStatus::Ready | RegionalBindingTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalBindingTierStatus::Ready => {
                return next == RegionalBindingTierStatus::Draining;
            }
            RegionalBindingTierStatus::Draining => {
                match next {
                    RegionalBindingTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalBindingTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional binding tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> RegionalBindingTier {
        RegionalBindingTier {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a regional binding tier in status idle.
pub fn new_regional_binding_tier(code: &str, label: &str) -> RegionalBindingTier {
    RegionalBindingTier {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        timezone: "".to_string(),
        ends_at: 0,
        checksum: "".to_string(),
        status: RegionalBindingTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_binding_tier("code-439", "label-417");
        assert_eq!(item.status, RegionalBindingTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_regional_binding_tier("code-439", "label-417");
        if item.move_to(RegionalBindingTierStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalBindingTierStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_binding_tier("code-439", "label-417");
        item.status = RegionalBindingTierStatus::Stopped;
        assert!(
            item.move_to(RegionalBindingTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
