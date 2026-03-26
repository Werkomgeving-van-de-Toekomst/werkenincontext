//! Filterpaneel: attribuutfilter op GeoJSON (EduGIS / Wikiwijs-workflow).

use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::layer_control_3d::{catalog_layers, CatalogLayer, LayerType};
use crate::components::map_3d::{build_apply_geo_filter_script, build_remove_layer_script, GeoFilterParams};

/// Selecteerbare GeoJSON-bronnen uit de kaartcatalogus.
#[derive(Clone, PartialEq)]
pub struct GeoJsonSourceOption {
    pub id: String,
    pub name: String,
    pub url: String,
    pub layer_type: String,
}

pub fn geojson_layer_options_from_catalog() -> Vec<GeoJsonSourceOption> {
    catalog_layers()
        .into_iter()
        .filter_map(|l| match l {
            CatalogLayer::Geojson {
                id,
                name,
                url,
                layer_type,
                ..
            } => Some(GeoJsonSourceOption {
                id,
                name,
                url,
                layer_type: match layer_type {
                    LayerType::Point => "point".to_string(),
                    LayerType::Line => "line".to_string(),
                    LayerType::Polygon => "polygon".to_string(),
                },
            }),
            CatalogLayer::Raster { .. } => None,
        })
        .collect()
}

fn is_valid_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Filter UI: invoerlaag, attribuut, operator, waarde, naam + kleur voor resultaatlaag.
#[component]
pub fn GeoFilterPanel(map_id: String) -> Element {
    let options = geojson_layer_options_from_catalog();
    let has_options = !options.is_empty();
    let mut selected_idx = use_signal(|| 0usize);
    let mut prop_key = use_signal(|| String::new());
    let mut op = use_signal(|| "contains".to_string());
    let mut value_str = use_signal(|| String::new());
    let mut result_label = use_signal(|| String::new());
    let mut color = use_signal(|| "#FF9800".to_string());
    let mut result_layers = use_signal(Vec::<(String, String)>::new);

    rsx! {
        div {
            class: "geo-filter-panel",
            style: "background: white; padding: 0.75rem; border-radius: 4px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.15); font-size: 0.85rem;
                    max-width: 280px;",

            h3 {
                style: "margin: 0 0 0.5rem 0; font-size: 0.95rem; font-weight: 600; color: #333;",
                "Filter → nieuwe laag"
            }
            p {
                style: "margin: 0 0 0.6rem 0; color: #555; line-height: 1.35;",
                "Zoals op Wikiwijs/EduGIS: filter op attributen en voeg het resultaat als aparte laag toe."
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Invoerlaag" }
            if has_options {
                select {
                    style: "width: 100%; margin-bottom: 0.5rem; padding: 4px;",
                    onchange: move |e: Event<FormData>| {
                        if let Ok(i) = e.value().parse::<usize>() {
                            selected_idx.set(i);
                        }
                    },
                    for (i, opt) in options.iter().enumerate() {
                        option { value: "{i}", selected: i == *selected_idx.read(), "{opt.name}" }
                    }
                }
            } else {
                p { style: "color: #c62828; margin-bottom: 0.5rem;", "Geen GeoJSON-lagen in de catalogus." }
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Attribuut (veldnaam)" }
            input {
                style: "width: 100%; margin-bottom: 0.5rem; padding: 4px; box-sizing: border-box;",
                placeholder: "bijv. NAAM of GEM_AANTAL_DIEREN",
                value: "{prop_key.read()}",
                oninput: move |e: Event<FormData>| prop_key.set(e.value()),
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Voorwaarde" }
            select {
                style: "width: 100%; margin-bottom: 0.5rem; padding: 4px;",
                onchange: move |e: Event<FormData>| op.set(e.value()),
                option { value: "contains", selected: *op.read() == "contains", "bevat (tekst)" }
                option { value: "eq", selected: *op.read() == "eq", "is gelijk aan (=)" }
                option { value: "gt", selected: *op.read() == "gt", "groter dan (>) — getallen" }
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Waarde" }
            input {
                style: "width: 100%; margin-bottom: 0.5rem; padding: 4px; box-sizing: border-box;",
                placeholder: "bijv. eenden of 10000",
                value: "{value_str.read()}",
                oninput: move |e: Event<FormData>| value_str.set(e.value()),
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Naam resultaatlaag" }
            input {
                style: "width: 100%; margin-bottom: 0.5rem; padding: 4px; box-sizing: border-box;",
                placeholder: "bijv. Alleen eenden",
                value: "{result_label.read()}",
                oninput: move |e: Event<FormData>| result_label.set(e.value()),
            }

            label { style: "display: block; margin-bottom: 0.2rem; color: #444;", "Kleur" }
            input {
                r#type: "color",
                style: "width: 100%; height: 28px; margin-bottom: 0.5rem; cursor: pointer;",
                value: "{color.read()}",
                oninput: move |e: Event<FormData>| {
                    let v = e.value();
                    if v.len() == 7 && v.starts_with('#') {
                        color.set(v);
                    }
                },
            }

            button {
                r#type: "button",
                disabled: !has_options,
                style: "width: 100%; padding: 8px; background: #1976D2; color: white; border: none;
                        border-radius: 4px; cursor: pointer; font-weight: 500;",
                onclick: move |_| {
                    if options.is_empty() {
                        return;
                    }
                    let idx = *selected_idx.read();
                    let Some(opt) = options.get(idx) else { return };
                    let pk = prop_key.read().trim().to_string();
                    let val = value_str.read().trim().to_string();
                    let lbl = result_label.read().trim().to_string();
                    let c = color.read().clone();
                    if pk.is_empty() || val.is_empty() || lbl.is_empty() {
                        return;
                    }
                    if !is_valid_hex_color(&c) {
                        return;
                    }
                    let result_id = format!("fl_{}", Uuid::new_v4().simple());
                    let params = GeoFilterParams {
                        url: opt.url.clone(),
                        layer_type: opt.layer_type.clone(),
                        prop_key: pk,
                        op: op.read().clone(),
                        value: val,
                        result_id: result_id.clone(),
                        color: c,
                    };
                    let script = build_apply_geo_filter_script(&params);
                    let _ = document::eval(&script);
                    result_layers.write().push((result_id, lbl));
                },
                "Filter toevoegen"
            }

            if !result_layers.read().is_empty() {
                div {
                    style: "margin-top: 0.75rem; padding-top: 0.5rem; border-top: 1px solid #eee;",
                    p { style: "margin: 0 0 0.35rem 0; font-weight: 600; color: #444;", "Resultaatlagen" }
                    for (lid, lname) in result_layers.read().clone() {
                        {
                            let lid_key = lid.clone();
                            let lid_rm = lid.clone();
                            let map_id_btn = map_id.clone();
                            rsx! {
                                div {
                                    key: "{lid_key}",
                                    style: "display: flex; align-items: center; justify-content: space-between; gap: 6px; margin: 0.25rem 0;",
                                    span { style: "overflow: hidden; text-overflow: ellipsis;", "{lname}" }
                                    button {
                                        r#type: "button",
                                        style: "flex-shrink: 0; padding: 2px 8px; font-size: 0.75rem; cursor: pointer;",
                                        onclick: move |_| {
                                            let rm = build_remove_layer_script(&map_id_btn, &lid_rm);
                                            let _ = document::eval(&rm);
                                            result_layers.write().retain(|(i, _)| i != &lid_rm);
                                        },
                                        "Verwijder"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::map_3d::GeoFilterParams;

    #[test]
    fn geo_filter_script_includes_ops() {
        let p = GeoFilterParams {
            url: "/x.geojson".to_string(),
            layer_type: "polygon".to_string(),
            prop_key: "a".to_string(),
            op: "contains".to_string(),
            value: "b".to_string(),
            result_id: "fl_test123".to_string(),
            color: "#ff0000".to_string(),
        };
        let s = build_apply_geo_filter_script(&p);
        assert!(s.contains("contains"));
        assert!(s.contains("eq"));
        assert!(s.contains("gt"));
        assert!(s.contains("fl_test123"));
    }
}
