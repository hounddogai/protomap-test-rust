use super::*;

use acme_kit::text;

/// Where a evidence tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceTierStatus {
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

impl std::fmt::Display for EvidenceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTier {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub pinned: bool,
    pub phone: String,
    pub approved: bool,
    pub rate: f64,
    pub visible: bool,
    pub priority: i64,
    pub quantity: i64,
    pub expires_at: i64,
    pub status: EvidenceTierStatus,
}

impl EvidenceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceTierStatus) -> bool {
        match self.status {
            EvidenceTierStatus::Idle => {
                match next {
                    EvidenceTierStatus::Ready | EvidenceTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceTierStatus::Warming => {
                match next {
                    EvidenceTierStatus::Draining | EvidenceTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceTierStatus::Ready => {
                match next {
                    EvidenceTierStatus::Draining | EvidenceTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceTierStatus::Draining => {
                match next {
                    EvidenceTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence tier {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.priority as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceTier {
        EvidenceTier {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            pinned: self.pinned,
            phone: self.phone.clone(),
            approved: self.approved,
            rate: self.rate,
            visible: self.visible,
            priority: self.priority,
            quantity: self.quantity,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence tier in status idle.
pub fn new_evidence_tier(code: &str, label: &str) -> EvidenceTier {
    EvidenceTier {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        pinned: false,
        phone: "".to_string(),
        approved: false,
        rate: 0.0,
        visible: false,
        priority: 0,
        quantity: 0,
        expires_at: 0,
        status: EvidenceTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_tier("code-576", "label-287");
        assert_eq!(item.status, EvidenceTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_evidence_tier("code-576", "label-287");
        if item.move_to(EvidenceTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_tier("code-576", "label-287");
        item.status = EvidenceTierStatus::Stopped;
        assert!(item.move_to(EvidenceTierStatus::Idle).is_err(), "expected move to fail");
    }
}
