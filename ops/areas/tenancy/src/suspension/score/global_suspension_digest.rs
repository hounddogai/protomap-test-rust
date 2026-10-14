use super::*;

use acme_kit::text;

/// Where a global suspension digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSuspensionDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalSuspensionDigestStatus {
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

impl std::fmt::Display for GlobalSuspensionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global suspension digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionDigest {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub limit: i64,
    pub owner_id: String,
    pub checksum: String,
    pub port: i64,
    pub status: GlobalSuspensionDigestStatus,
}

impl GlobalSuspensionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSuspensionDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSuspensionDigestStatus) -> bool {
        match self.status {
            GlobalSuspensionDigestStatus::Idle => {
                match next {
                    GlobalSuspensionDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSuspensionDigestStatus::Warming => {
                match next {
                    GlobalSuspensionDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSuspensionDigestStatus::Ready => {
                return next == GlobalSuspensionDigestStatus::Draining || next == GlobalSuspensionDigestStatus::Stopped;
            }
            GlobalSuspensionDigestStatus::Draining => {
                match next {
                    GlobalSuspensionDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalSuspensionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global suspension digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSuspensionDigest {
        GlobalSuspensionDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global suspension digest in status idle.
pub fn new_global_suspension_digest(code: &str, title: &str) -> GlobalSuspensionDigest {
    GlobalSuspensionDigest {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        limit: 0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        port: 0,
        status: GlobalSuspensionDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_suspension_digest("code-444", "title-437");
        assert_eq!(item.status, GlobalSuspensionDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_global_suspension_digest("code-444", "title-437");
        if item.move_to(GlobalSuspensionDigestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSuspensionDigestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_suspension_digest("code-444", "title-437");
        item.status = GlobalSuspensionDigestStatus::Stopped;
        assert!(
            item.move_to(GlobalSuspensionDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
