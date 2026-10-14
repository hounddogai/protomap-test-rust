use super::*;

use acme_kit::text;

/// Where a federation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationDigestStatus {
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

impl std::fmt::Display for FederationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation digest as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigest {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub batch_size: i64,
    pub starts_at: i64,
    pub name: String,
    pub ratio: f64,
    pub audience: String,
    pub host: String,
    pub phone: String,
    pub status: FederationDigestStatus,
}

impl FederationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationDigestStatus) -> bool {
        match self.status {
            FederationDigestStatus::Idle => {
                return next == FederationDigestStatus::Warming;
            }
            FederationDigestStatus::Warming => {
                match next {
                    FederationDigestStatus::Ready | FederationDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationDigestStatus::Ready => {
                return next == FederationDigestStatus::Stopped;
            }
            FederationDigestStatus::Draining => {
                match next {
                    FederationDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FederationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationDigest {
        FederationDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            name: self.name.clone(),
            ratio: self.ratio,
            audience: self.audience.clone(),
            host: self.host.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a federation digest in status idle.
pub fn new_federation_digest(code: &str, label: &str) -> FederationDigest {
    FederationDigest {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        batch_size: 0,
        starts_at: 0,
        name: "".to_string(),
        ratio: 0.0,
        audience: "".to_string(),
        host: "".to_string(),
        phone: "".to_string(),
        status: FederationDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_federation_digest("code-653", "label-437");
        assert_eq!(item.status, FederationDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_federation_digest("code-653", "label-437");
        if item.move_to(FederationDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_federation_digest("code-653", "label-437");
        item.status = FederationDigestStatus::Stopped;
        assert!(
            item.move_to(FederationDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
