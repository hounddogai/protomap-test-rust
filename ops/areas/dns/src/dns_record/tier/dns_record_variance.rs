use super::*;

use acme_kit::text;

/// Where a dns record variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordVarianceStatus {
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

impl std::fmt::Display for DnsRecordVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVariance {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub ratio: f64,
    pub summary: String,
    pub label: String,
    pub cipher: String,
    pub visible: bool,
    pub cost: f64,
    pub maximum: f64,
    pub status: DnsRecordVarianceStatus,
}

impl DnsRecordVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordVarianceStatus) -> bool {
        match self.status {
            DnsRecordVarianceStatus::Draft => {
                match next {
                    DnsRecordVarianceStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordVarianceStatus::Pending => {
                match next {
                    DnsRecordVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordVarianceStatus::Active => {
                match next {
                    DnsRecordVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordVarianceStatus::Suspended => {
                match next {
                    DnsRecordVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: DnsRecordVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.cost * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordVariance {
        DnsRecordVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            ratio: self.ratio,
            summary: self.summary.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            visible: self.visible,
            cost: self.cost,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record variance in status draft.
pub fn new_dns_record_variance(code: &str, title: &str) -> DnsRecordVariance {
    DnsRecordVariance {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        ratio: 0.0,
        summary: "".to_string(),
        label: "".to_string(),
        cipher: "".to_string(),
        visible: false,
        cost: 0.0,
        maximum: 0.0,
        status: DnsRecordVarianceStatus::Draft,
    }
}
