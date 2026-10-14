use super::*;

use acme_kit::text;

/// How screens and exports show a canary cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCycleView {
    pub code_text: String,
    pub name: String,
    pub ceiling: String,
    pub phone: String,
    pub notes: String,
    pub tenant_id: String,
    pub port: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_canary_cycle_view(item: &CanaryCycle) -> CanaryCycleView {
    CanaryCycleView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        phone: item.phone.clone(),
        notes: item.notes.clone(),
        tenant_id: item.tenant_id.clone(),
        port: item.port.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_cycle_views(items: &[CanaryCycle]) -> Vec<CanaryCycleView> {
    let mut views: Vec<CanaryCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_cycle_view(item));
    }
    views
}

pub fn sum_canary_cycle_view(view: &CanaryCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ceiling + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_cycle("code-791", "name-329");
        let view = to_canary_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
