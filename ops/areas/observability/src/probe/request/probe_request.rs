use super::*;

use acme_kit::text;

/// Where a probe request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeRequestStatus {
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

impl std::fmt::Display for ProbeRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequest {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub ceiling: f64,
    pub cluster: String,
    pub weight: f64,
    pub reviewer: String,
    pub ratio: f64,
    pub status: ProbeRequestStatus,
}

impl ProbeRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeRequestStatus) -> bool {
        match self.status {
            ProbeRequestStatus::Draft => {
                match next {
                    ProbeRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRequestStatus::Pending => {
                match next {
                    ProbeRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRequestStatus::Active => {
                match next {
                    ProbeRequestStatus::Suspended | ProbeRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeRequestStatus::Suspended => {
                return next == ProbeRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe request", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        (self.score * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeRequest {
        ProbeRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a probe request in status draft.
pub fn new_probe_request(code: &str, label: &str) -> ProbeRequest {
    ProbeRequest {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        ceiling: 0.0,
        cluster: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        ratio: 0.0,
        status: ProbeRequestStatus::Draft,
    }
}
