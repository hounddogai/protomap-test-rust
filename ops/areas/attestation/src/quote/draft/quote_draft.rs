use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a quote draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteDraftStatus {
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

impl std::fmt::Display for QuoteDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDraft {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub quota_units: i64,
    pub margin: f64,
    pub attempts: i64,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub weight: f64,
    pub ceiling: f64,
    pub status: QuoteDraftStatus,
}

impl QuoteDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteDraftStatus) -> bool {
        match self.status {
            QuoteDraftStatus::Idle => {
                return next == QuoteDraftStatus::Warming;
            }
            QuoteDraftStatus::Warming => {
                return next == QuoteDraftStatus::Draining || next == QuoteDraftStatus::Stopped;
            }
            QuoteDraftStatus::Ready => {
                match next {
                    QuoteDraftStatus::Draining | QuoteDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteDraftStatus::Draining => {
                match next {
                    QuoteDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.ceiling + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteDraft {
        QuoteDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            quota_units: self.quota_units,
            margin: self.margin,
            attempts: self.attempts,
            attributes: self.attributes.clone(),
            visible: self.visible,
            weight: self.weight,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote draft in status idle.
pub fn new_quote_draft(code: &str, name: &str) -> QuoteDraft {
    QuoteDraft {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        quota_units: 0,
        margin: 0.0,
        attempts: 0,
        attributes: BTreeMap::new(),
        visible: false,
        weight: 0.0,
        ceiling: 0.0,
        status: QuoteDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_draft("code-67", "name-566");
        assert_eq!(item.status, QuoteDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_quote_draft("code-67", "name-566");
        if item.move_to(QuoteDraftStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteDraftStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_draft("code-67", "name-566");
        item.status = QuoteDraftStatus::Stopped;
        assert!(item.move_to(QuoteDraftStatus::Idle).is_err(), "expected move to fail");
    }
}
