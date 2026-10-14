use super::*;

use acme_kit::text;

/// Where a probe segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeSegmentStatus {
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

impl std::fmt::Display for ProbeSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe segment as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegment {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub title: String,
    pub status: ProbeSegmentStatus,
}

impl ProbeSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeSegmentStatus) -> bool {
        match self.status {
            ProbeSegmentStatus::Draft => {
                match next {
                    ProbeSegmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeSegmentStatus::Pending => {
                return next == ProbeSegmentStatus::Suspended || next == ProbeSegmentStatus::Closed;
            }
            ProbeSegmentStatus::Active => {
                match next {
                    ProbeSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeSegmentStatus::Suspended => {
                return next == ProbeSegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe segment", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.created_at as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeSegment {
        ProbeSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            created_at: self.created_at,
            expires_at: self.expires_at,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe segment in status draft.
pub fn new_probe_segment(code: &str, label: &str) -> ProbeSegment {
    ProbeSegment {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        created_at: 0,
        expires_at: 0,
        title: "".to_string(),
        status: ProbeSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_segment("code-561", "label-564");
        assert_eq!(item.status, ProbeSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_probe_segment("code-561", "label-564");
        if item.move_to(ProbeSegmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeSegmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_segment("code-561", "label-564");
        item.status = ProbeSegmentStatus::Closed;
        assert!(
            item.move_to(ProbeSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
