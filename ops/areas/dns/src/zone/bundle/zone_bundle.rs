use super::*;

use acme_kit::text;

/// Where a zone bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ZoneBundleStatus {
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

impl std::fmt::Display for ZoneBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBundle {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub window_days: i64,
    pub ratio: f64,
    pub name: String,
    pub source: String,
    pub contact_email: String,
    pub currency: String,
    pub status: ZoneBundleStatus,
}

impl ZoneBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneBundleStatus) -> bool {
        match self.status {
            ZoneBundleStatus::Draft => {
                return next == ZoneBundleStatus::Pending;
            }
            ZoneBundleStatus::Pending => {
                match next {
                    ZoneBundleStatus::Suspended | ZoneBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneBundleStatus::Active => {
                return next == ZoneBundleStatus::Suspended || next == ZoneBundleStatus::Closed;
            }
            ZoneBundleStatus::Suspended => {
                return next == ZoneBundleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneBundle {
        ZoneBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            window_days: self.window_days,
            ratio: self.ratio,
            name: self.name.clone(),
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a zone bundle in status draft.
pub fn new_zone_bundle(code: &str, label: &str) -> ZoneBundle {
    ZoneBundle {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        window_days: 0,
        ratio: 0.0,
        name: "".to_string(),
        source: "".to_string(),
        contact_email: "".to_string(),
        currency: "".to_string(),
        status: ZoneBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_zone_bundle("code-582", "label-183");
        assert_eq!(item.status, ZoneBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_zone_bundle("code-582", "label-183");
        if item.move_to(ZoneBundleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneBundleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_zone_bundle("code-582", "label-183");
        item.status = ZoneBundleStatus::Closed;
        assert!(item.move_to(ZoneBundleStatus::Draft).is_err(), "expected move to fail");
    }
}
