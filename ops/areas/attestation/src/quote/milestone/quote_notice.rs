use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a quote notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QuoteNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QuoteNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote notice as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteNotice {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub ceiling: f64,
    pub tenant_id: String,
    pub status: QuoteNoticeStatus,
}

impl QuoteNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteNoticeStatus) -> bool {
        match self.status {
            QuoteNoticeStatus::Created => {
                match next {
                    QuoteNoticeStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteNoticeStatus::Validated => {
                return next == QuoteNoticeStatus::Retired;
            }
            QuoteNoticeStatus::Published => {
                match next {
                    QuoteNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: QuoteNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> QuoteNotice {
        QuoteNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote notice in status created.
pub fn new_quote_notice(code: &str, name: &str) -> QuoteNotice {
    QuoteNotice {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        pinned: false,
        ceiling: 0.0,
        tenant_id: "".to_string(),
        status: QuoteNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_quote_notice("code-481", "name-271");
        assert_eq!(item.status, QuoteNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_quote_notice("code-481", "name-271");
        if item.move_to(QuoteNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_quote_notice("code-481", "name-271");
        item.status = QuoteNoticeStatus::Retired;
        assert!(
            item.move_to(QuoteNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
