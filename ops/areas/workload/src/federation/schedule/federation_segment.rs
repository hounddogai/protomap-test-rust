use super::*;

use acme_kit::text;

/// Where a federation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationSegmentStatus {
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

impl std::fmt::Display for FederationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation segment as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationSegment {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub description: String,
    pub cipher: String,
    pub expires_at: i64,
    pub namespace: String,
    pub region: String,
    pub grace_days: i64,
    pub status: FederationSegmentStatus,
}

impl FederationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationSegmentStatus) -> bool {
        match self.status {
            FederationSegmentStatus::Idle => {
                return next == FederationSegmentStatus::Ready || next == FederationSegmentStatus::Draining;
            }
            FederationSegmentStatus::Warming => {
                match next {
                    FederationSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationSegmentStatus::Ready => {
                return next == FederationSegmentStatus::Draining || next == FederationSegmentStatus::Stopped;
            }
            FederationSegmentStatus::Draining => {
                return next == FederationSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation segment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationSegment {
        FederationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            description: self.description.clone(),
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            region: self.region.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a federation segment in status idle.
pub fn new_federation_segment(code: &str, name: &str) -> FederationSegment {
    FederationSegment {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        description: "".to_string(),
        cipher: "".to_string(),
        expires_at: 0,
        namespace: "".to_string(),
        region: "".to_string(),
        grace_days: 0,
        status: FederationSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_federation_segment("code-156", "name-823");
        assert_eq!(item.status, FederationSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_federation_segment("code-156", "name-823");
        if item.move_to(FederationSegmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationSegmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_federation_segment("code-156", "name-823");
        item.status = FederationSegmentStatus::Stopped;
        assert!(
            item.move_to(FederationSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
