use super::*;

use acme_kit::text;

/// Where a lenient redaction policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRedactionPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientRedactionPolicyStatus {
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

impl std::fmt::Display for LenientRedactionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient redaction policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionPolicy {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub created_at: i64,
    pub archived: bool,
    pub pinned: bool,
    pub ends_at: i64,
    pub description: String,
    pub port: i64,
    pub status: LenientRedactionPolicyStatus,
}

impl LenientRedactionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRedactionPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRedactionPolicyStatus) -> bool {
        match self.status {
            LenientRedactionPolicyStatus::Idle => {
                match next {
                    LenientRedactionPolicyStatus::Ready | LenientRedactionPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionPolicyStatus::Warming => {
                return next == LenientRedactionPolicyStatus::Ready || next == LenientRedactionPolicyStatus::Stopped;
            }
            LenientRedactionPolicyStatus::Ready => {
                match next {
                    LenientRedactionPolicyStatus::Draining | LenientRedactionPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionPolicyStatus::Draining => {
                match next {
                    LenientRedactionPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientRedactionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient redaction policy {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ends_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRedactionPolicy {
        LenientRedactionPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            created_at: self.created_at,
            archived: self.archived,
            pinned: self.pinned,
            ends_at: self.ends_at,
            description: self.description.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient redaction policy in status idle.
pub fn new_lenient_redaction_policy(code: &str, name: &str) -> LenientRedactionPolicy {
    LenientRedactionPolicy {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        created_at: 0,
        archived: false,
        pinned: false,
        ends_at: 0,
        description: "".to_string(),
        port: 0,
        status: LenientRedactionPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_redaction_policy("code-878", "name-653");
        assert_eq!(item.status, LenientRedactionPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_redaction_policy("code-878", "name-653");
        if item.move_to(LenientRedactionPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRedactionPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_redaction_policy("code-878", "name-653");
        item.status = LenientRedactionPolicyStatus::Stopped;
        assert!(
            item.move_to(LenientRedactionPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
