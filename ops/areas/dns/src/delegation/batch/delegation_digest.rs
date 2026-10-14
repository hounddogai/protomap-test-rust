use super::*;

use acme_kit::text;

/// Where a delegation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DelegationDigestStatus {
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

impl std::fmt::Display for DelegationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigest {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub rate: f64,
    pub visible: bool,
    pub checksum: String,
    pub pinned: bool,
    pub burst: i64,
    pub maximum: f64,
    pub status: DelegationDigestStatus,
}

impl DelegationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationDigestStatus) -> bool {
        match self.status {
            DelegationDigestStatus::Idle => {
                return next == DelegationDigestStatus::Warming || next == DelegationDigestStatus::Stopped;
            }
            DelegationDigestStatus::Warming => {
                match next {
                    DelegationDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationDigestStatus::Ready => {
                match next {
                    DelegationDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationDigestStatus::Draining => {
                match next {
                    DelegationDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DelegationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.maximum + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationDigest {
        DelegationDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            rate: self.rate,
            visible: self.visible,
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            burst: self.burst,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a delegation digest in status idle.
pub fn new_delegation_digest(code: &str, title: &str) -> DelegationDigest {
    DelegationDigest {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        rate: 0.0,
        visible: false,
        checksum: "".to_string(),
        pinned: false,
        burst: 0,
        maximum: 0.0,
        status: DelegationDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_delegation_digest("code-737", "title-357");
        assert_eq!(item.status, DelegationDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_delegation_digest("code-737", "title-357");
        if item.move_to(DelegationDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_delegation_digest("code-737", "title-357");
        item.status = DelegationDigestStatus::Stopped;
        assert!(
            item.move_to(DelegationDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
