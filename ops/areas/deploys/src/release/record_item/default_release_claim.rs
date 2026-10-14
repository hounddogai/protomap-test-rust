use super::*;

use acme_kit::text;

/// Where a default release claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultReleaseClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DefaultReleaseClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default release claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaim {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub priority: i64,
    pub expires_at: i64,
    pub status: DefaultReleaseClaimStatus,
}

impl DefaultReleaseClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseClaimStatus) -> bool {
        match self.status {
            DefaultReleaseClaimStatus::New => {
                match next {
                    DefaultReleaseClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseClaimStatus::Queued => {
                return next == DefaultReleaseClaimStatus::Done || next == DefaultReleaseClaimStatus::Failed;
            }
            DefaultReleaseClaimStatus::Running => {
                match next {
                    DefaultReleaseClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseClaimStatus::Done => {
                match next {
                    DefaultReleaseClaimStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultReleaseClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default release claim", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseClaim {
        DefaultReleaseClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            priority: self.priority,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default release claim in status new.
pub fn new_default_release_claim(code: &str, label: &str) -> DefaultReleaseClaim {
    DefaultReleaseClaim {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        priority: 0,
        expires_at: 0,
        status: DefaultReleaseClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_release_claim("code-257", "label-182");
        assert_eq!(item.status, DefaultReleaseClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_default_release_claim("code-257", "label-182");
        if item.move_to(DefaultReleaseClaimStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultReleaseClaimStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_release_claim("code-257", "label-182");
        item.status = DefaultReleaseClaimStatus::Failed;
        assert!(
            item.move_to(DefaultReleaseClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
