use super::*;

use acme_kit::text;

/// Where a lenient suspension assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSuspensionAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientSuspensionAssignmentStatus {
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

impl std::fmt::Display for LenientSuspensionAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient suspension assignment as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignment {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub reviewer: String,
    pub unit: String,
    pub channel: String,
    pub pinned: bool,
    pub status: LenientSuspensionAssignmentStatus,
}

impl LenientSuspensionAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSuspensionAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSuspensionAssignmentStatus) -> bool {
        match self.status {
            LenientSuspensionAssignmentStatus::Draft => {
                return next == LenientSuspensionAssignmentStatus::Pending
                    || next == LenientSuspensionAssignmentStatus::Active;
            }
            LenientSuspensionAssignmentStatus::Pending => {
                match next {
                    LenientSuspensionAssignmentStatus::Active | LenientSuspensionAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionAssignmentStatus::Active => {
                match next {
                    LenientSuspensionAssignmentStatus::Suspended | LenientSuspensionAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionAssignmentStatus::Suspended => {
                return next == LenientSuspensionAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSuspensionAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient suspension assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ceiling - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSuspensionAssignment {
        LenientSuspensionAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            reviewer: self.reviewer.clone(),
            unit: self.unit.clone(),
            channel: self.channel.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient suspension assignment in status draft.
pub fn new_lenient_suspension_assignment(code: &str, title: &str) -> LenientSuspensionAssignment {
    LenientSuspensionAssignment {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        ceiling: 0.0,
        reviewer: "".to_string(),
        unit: "".to_string(),
        channel: "".to_string(),
        pinned: false,
        status: LenientSuspensionAssignmentStatus::Draft,
    }
}
