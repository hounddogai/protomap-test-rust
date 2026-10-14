use super::*;

use acme_kit::text;

/// Where a slo group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SloGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo group as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloGroup {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub discount: f64,
    pub batch_size: i64,
    pub namespace: String,
    pub cipher: String,
    pub port: i64,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub status: SloGroupStatus,
}

impl SloGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloGroupStatus) -> bool {
        match self.status {
            SloGroupStatus::Planned => {
                return next == SloGroupStatus::InProgress || next == SloGroupStatus::Cancelled;
            }
            SloGroupStatus::Scheduled => {
                match next {
                    SloGroupStatus::Completed | SloGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloGroupStatus::InProgress => {
                return next == SloGroupStatus::Completed || next == SloGroupStatus::Cancelled;
            }
            SloGroupStatus::Completed => {
                return next == SloGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.quantity as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloGroup {
        SloGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            discount: self.discount,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            cipher: self.cipher.clone(),
            port: self.port,
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a slo group in status planned.
pub fn new_slo_group(code: &str, label: &str) -> SloGroup {
    SloGroup {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        discount: 0.0,
        batch_size: 0,
        namespace: "".to_string(),
        cipher: "".to_string(),
        port: 0,
        quantity: 0,
        duration_minutes: 0,
        status: SloGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_group("code-948", "label-918");
        assert_eq!(item.status, SloGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_slo_group("code-948", "label-918");
        if item.move_to(SloGroupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloGroupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_group("code-948", "label-918");
        item.status = SloGroupStatus::Cancelled;
        assert!(item.move_to(SloGroupStatus::Planned).is_err(), "expected move to fail");
    }
}
