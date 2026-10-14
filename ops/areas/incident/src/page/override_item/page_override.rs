use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a page override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PageOverrideStatus {
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

impl std::fmt::Display for PageOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageOverride {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub key_id: String,
    pub ratio: f64,
    pub region: String,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub updated_at: i64,
    pub status: PageOverrideStatus,
}

impl PageOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageOverrideStatus) -> bool {
        match self.status {
            PageOverrideStatus::Idle => {
                match next {
                    PageOverrideStatus::Warming | PageOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageOverrideStatus::Warming => {
                match next {
                    PageOverrideStatus::Draining | PageOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageOverrideStatus::Ready => {
                return next == PageOverrideStatus::Draining || next == PageOverrideStatus::Stopped;
            }
            PageOverrideStatus::Draining => {
                match next {
                    PageOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PageOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.ratio + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageOverride {
        PageOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            ratio: self.ratio,
            region: self.region.clone(),
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a page override in status idle.
pub fn new_page_override(code: &str, label: &str) -> PageOverride {
    PageOverride {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        key_id: "".to_string(),
        ratio: 0.0,
        region: "".to_string(),
        attributes: BTreeMap::new(),
        quantity: 0,
        updated_at: 0,
        status: PageOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_page_override("code-149", "label-528");
        assert_eq!(item.status, PageOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_page_override("code-149", "label-528");
        if item.move_to(PageOverrideStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageOverrideStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_page_override("code-149", "label-528");
        item.status = PageOverrideStatus::Stopped;
        assert!(item.move_to(PageOverrideStatus::Idle).is_err(), "expected move to fail");
    }
}
