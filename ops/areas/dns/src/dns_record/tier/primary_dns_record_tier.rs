use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary dns record tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDnsRecordTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryDnsRecordTierStatus {
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

impl std::fmt::Display for PrimaryDnsRecordTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary dns record tier as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTier {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub approved: bool,
    pub visible: bool,
    pub attributes: BTreeMap<String, String>,
    pub issuer: String,
    pub margin: f64,
    pub host: String,
    pub status: PrimaryDnsRecordTierStatus,
}

impl PrimaryDnsRecordTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDnsRecordTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDnsRecordTierStatus) -> bool {
        match self.status {
            PrimaryDnsRecordTierStatus::Idle => {
                match next {
                    PrimaryDnsRecordTierStatus::Ready | PrimaryDnsRecordTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordTierStatus::Warming => {
                return next == PrimaryDnsRecordTierStatus::Ready;
            }
            PrimaryDnsRecordTierStatus::Ready => {
                match next {
                    PrimaryDnsRecordTierStatus::Draining | PrimaryDnsRecordTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordTierStatus::Draining => {
                return next == PrimaryDnsRecordTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDnsRecordTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dns record tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary dns record tier", self.code)
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

    pub fn net_value(&self) -> f64 {
        (self.margin * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDnsRecordTier {
        PrimaryDnsRecordTier {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            approved: self.approved,
            visible: self.visible,
            attributes: self.attributes.clone(),
            issuer: self.issuer.clone(),
            margin: self.margin,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary dns record tier in status idle.
pub fn new_primary_dns_record_tier(code: &str, label: &str) -> PrimaryDnsRecordTier {
    PrimaryDnsRecordTier {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        approved: false,
        visible: false,
        attributes: BTreeMap::new(),
        issuer: "".to_string(),
        margin: 0.0,
        host: "".to_string(),
        status: PrimaryDnsRecordTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_dns_record_tier("code-844", "label-992");
        assert_eq!(item.status, PrimaryDnsRecordTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_dns_record_tier("code-844", "label-992");
        if item.move_to(PrimaryDnsRecordTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDnsRecordTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_dns_record_tier("code-844", "label-992");
        item.status = PrimaryDnsRecordTierStatus::Stopped;
        assert!(
            item.move_to(PrimaryDnsRecordTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
