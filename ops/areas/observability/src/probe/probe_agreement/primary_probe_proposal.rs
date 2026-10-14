use super::*;

use acme_kit::text;

/// Where a primary probe proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryProbeProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryProbeProposalStatus {
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

impl std::fmt::Display for PrimaryProbeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary probe proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposal {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub country: String,
    pub region: String,
    pub minimum: f64,
    pub expires_at: i64,
    pub window_days: i64,
    pub fingerprint: String,
    pub rate: f64,
    pub status: PrimaryProbeProposalStatus,
}

impl PrimaryProbeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryProbeProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryProbeProposalStatus) -> bool {
        match self.status {
            PrimaryProbeProposalStatus::Idle => {
                match next {
                    PrimaryProbeProposalStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryProbeProposalStatus::Warming => {
                return next == PrimaryProbeProposalStatus::Ready || next == PrimaryProbeProposalStatus::Draining;
            }
            PrimaryProbeProposalStatus::Ready => {
                return next == PrimaryProbeProposalStatus::Draining || next == PrimaryProbeProposalStatus::Stopped;
            }
            PrimaryProbeProposalStatus::Draining => {
                return next == PrimaryProbeProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryProbeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary probe proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.priority as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryProbeProposal {
        PrimaryProbeProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            country: self.country.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            expires_at: self.expires_at,
            window_days: self.window_days,
            fingerprint: self.fingerprint.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary probe proposal in status idle.
pub fn new_primary_probe_proposal(code: &str, label: &str) -> PrimaryProbeProposal {
    PrimaryProbeProposal {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        country: "".to_string(),
        region: "".to_string(),
        minimum: 0.0,
        expires_at: 0,
        window_days: 0,
        fingerprint: "".to_string(),
        rate: 0.0,
        status: PrimaryProbeProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_probe_proposal("code-901", "label-200");
        assert_eq!(item.status, PrimaryProbeProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_primary_probe_proposal("code-901", "label-200");
        if item.move_to(PrimaryProbeProposalStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryProbeProposalStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_probe_proposal("code-901", "label-200");
        item.status = PrimaryProbeProposalStatus::Stopped;
        assert!(
            item.move_to(PrimaryProbeProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
