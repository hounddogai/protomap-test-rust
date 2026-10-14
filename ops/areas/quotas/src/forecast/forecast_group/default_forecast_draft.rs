use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default forecast draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultForecastDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultForecastDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DefaultForecastDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default forecast draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastDraft {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub attempts: i64,
    pub source: String,
    pub discount: f64,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub margin: f64,
    pub active: bool,
    pub status: DefaultForecastDraftStatus,
}

impl DefaultForecastDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultForecastDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultForecastDraftStatus) -> bool {
        match self.status {
            DefaultForecastDraftStatus::Draft => {
                match next {
                    DefaultForecastDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastDraftStatus::Pending => {
                return next == DefaultForecastDraftStatus::Active;
            }
            DefaultForecastDraftStatus::Active => {
                return next == DefaultForecastDraftStatus::Suspended || next == DefaultForecastDraftStatus::Closed;
            }
            DefaultForecastDraftStatus::Suspended => {
                match next {
                    DefaultForecastDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultForecastDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default forecast draft {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
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

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultForecastDraft {
        DefaultForecastDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            attempts: self.attempts,
            source: self.source.clone(),
            discount: self.discount,
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            margin: self.margin,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default forecast draft in status draft.
pub fn new_default_forecast_draft(code: &str, name: &str) -> DefaultForecastDraft {
    DefaultForecastDraft {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        attempts: 0,
        source: "".to_string(),
        discount: 0.0,
        attributes: BTreeMap::new(),
        expires_at: 0,
        margin: 0.0,
        active: false,
        status: DefaultForecastDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_forecast_draft("code-870", "name-67");
        assert_eq!(item.status, DefaultForecastDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_default_forecast_draft("code-870", "name-67");
        if item.move_to(DefaultForecastDraftStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultForecastDraftStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_forecast_draft("code-870", "name-67");
        item.status = DefaultForecastDraftStatus::Closed;
        assert!(
            item.move_to(DefaultForecastDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
