use super::*;

use acme_kit::text;

/// Where a slo tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloTierStatus {
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

impl std::fmt::Display for SloTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTier {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub region: String,
    pub limit: i64,
    pub cluster: String,
    pub scope: String,
    pub quota_units: i64,
    pub status: SloTierStatus,
}

impl SloTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloTierStatus) -> bool {
        match self.status {
            SloTierStatus::Idle => {
                return next == SloTierStatus::Warming;
            }
            SloTierStatus::Warming => {
                match next {
                    SloTierStatus::Ready | SloTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloTierStatus::Ready => {
                return next == SloTierStatus::Stopped;
            }
            SloTierStatus::Draining => {
                return next == SloTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo tier", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloTier {
        SloTier {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            region: self.region.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo tier in status idle.
pub fn new_slo_tier(code: &str, label: &str) -> SloTier {
    SloTier {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        region: "".to_string(),
        limit: 0,
        cluster: "".to_string(),
        scope: "".to_string(),
        quota_units: 0,
        status: SloTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_slo_tier("code-768", "label-612");
        assert_eq!(item.status, SloTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_slo_tier("code-768", "label-612");
        if item.move_to(SloTierStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloTierStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_slo_tier("code-768", "label-612");
        item.status = SloTierStatus::Stopped;
        assert!(item.move_to(SloTierStatus::Idle).is_err(), "expected move to fail");
    }
}
