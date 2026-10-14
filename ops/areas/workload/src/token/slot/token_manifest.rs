use super::*;

use acme_kit::text;

/// Where a token manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenManifestStatus {
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

impl std::fmt::Display for TokenManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenManifest {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub source: String,
    pub priority: i64,
    pub status: TokenManifestStatus,
}

impl TokenManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenManifestStatus) -> bool {
        match self.status {
            TokenManifestStatus::Idle => {
                match next {
                    TokenManifestStatus::Warming | TokenManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenManifestStatus::Warming => {
                match next {
                    TokenManifestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenManifestStatus::Ready => {
                match next {
                    TokenManifestStatus::Draining | TokenManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenManifestStatus::Draining => {
                return next == TokenManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.priority as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenManifest {
        TokenManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            source: self.source.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a token manifest in status idle.
pub fn new_token_manifest(code: &str, name: &str) -> TokenManifest {
    TokenManifest {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        source: "".to_string(),
        priority: 0,
        status: TokenManifestStatus::Idle,
    }
}
