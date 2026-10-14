use super::*;

use acme_kit::text;

/// Where a token bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenBundleStatus {
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

impl std::fmt::Display for TokenBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundle {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub retries: i64,
    pub title: String,
    pub phone: String,
    pub floor_value: f64,
    pub source: String,
    pub active: bool,
    pub status: TokenBundleStatus,
}

impl TokenBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenBundleStatus) -> bool {
        match self.status {
            TokenBundleStatus::Idle => {
                return next == TokenBundleStatus::Warming;
            }
            TokenBundleStatus::Warming => {
                match next {
                    TokenBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenBundleStatus::Ready => {
                return next == TokenBundleStatus::Draining;
            }
            TokenBundleStatus::Draining => {
                match next {
                    TokenBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TokenBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token bundle {} cannot move from {} to {}",
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

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenBundle {
        TokenBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            title: self.title.clone(),
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            source: self.source.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a token bundle in status idle.
pub fn new_token_bundle(code: &str, label: &str) -> TokenBundle {
    TokenBundle {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        retries: 0,
        title: "".to_string(),
        phone: "".to_string(),
        floor_value: 0.0,
        source: "".to_string(),
        active: false,
        status: TokenBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_bundle("code-899", "label-167");
        assert_eq!(item.status, TokenBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_token_bundle("code-899", "label-167");
        if item.move_to(TokenBundleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenBundleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_bundle("code-899", "label-167");
        item.status = TokenBundleStatus::Stopped;
        assert!(item.move_to(TokenBundleStatus::Idle).is_err(), "expected move to fail");
    }
}
