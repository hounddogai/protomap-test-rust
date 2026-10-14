use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback nonce estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackNonceEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackNonceEstimateStatus {
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

impl std::fmt::Display for FallbackNonceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback nonce estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceEstimate {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub limit: i64,
    pub status: FallbackNonceEstimateStatus,
}

impl FallbackNonceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackNonceEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackNonceEstimateStatus) -> bool {
        match self.status {
            FallbackNonceEstimateStatus::Idle => {
                match next {
                    FallbackNonceEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceEstimateStatus::Warming => {
                match next {
                    FallbackNonceEstimateStatus::Draining | FallbackNonceEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceEstimateStatus::Ready => {
                match next {
                    FallbackNonceEstimateStatus::Draining | FallbackNonceEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceEstimateStatus::Draining => {
                match next {
                    FallbackNonceEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackNonceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback nonce estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback nonce estimate", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_load(&self) -> f64 {
        ((self.quantity as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackNonceEstimate {
        FallbackNonceEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback nonce estimate in status idle.
pub fn new_fallback_nonce_estimate(code: &str, title: &str) -> FallbackNonceEstimate {
    FallbackNonceEstimate {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        attributes: BTreeMap::new(),
        quantity: 0,
        limit: 0,
        status: FallbackNonceEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_nonce_estimate("code-612", "title-530");
        assert_eq!(item.status, FallbackNonceEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_fallback_nonce_estimate("code-612", "title-530");
        if item.move_to(FallbackNonceEstimateStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackNonceEstimateStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_nonce_estimate("code-612", "title-530");
        item.status = FallbackNonceEstimateStatus::Stopped;
        assert!(
            item.move_to(FallbackNonceEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
