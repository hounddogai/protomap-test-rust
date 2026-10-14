use super::*;

use acme_kit::text;

/// How screens and exports show a regional delegation template.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateView {
    pub code: String,
    pub name_text: String,
    pub ceiling: String,
    pub ends_at_text: String,
    pub created_at_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_regional_delegation_template_view(item: &RegionalDelegationTemplate) -> RegionalDelegationTemplateView {
    RegionalDelegationTemplateView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 120),
        ceiling: format!("{:.2}", item.ceiling),
        ends_at_text: item.ends_at.to_string(),
        created_at_text: item.created_at.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_delegation_template_views(
    items: &[RegionalDelegationTemplate],
) -> Vec<RegionalDelegationTemplateView> {
    let mut views: Vec<RegionalDelegationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_delegation_template_view(item));
    }
    views
}

pub fn sum_regional_delegation_template_view(view: &RegionalDelegationTemplateView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ceiling + (parsed_ends_at as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_delegation_template("code-897", "name-905");
        let view = to_regional_delegation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_delegation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
