use super::*;

fn imports(map: &ImportMap) -> serde_json::Value {
    let html = map.to_string();
    let json = html
        .strip_prefix(r#"<script type="importmap">"#)
        .and_then(|h| h.strip_suffix("</script>"))
        .expect("one script element");
    serde_json::from_str::<serde_json::Value>(json).expect("JSON")["imports"].clone()
}

#[test]
fn names_three_its_addons_the_hook_and_datastar_under_the_bundle() {
    let base = base("/three");
    assert_eq!(
        imports(&ImportMap::new("/three")),
        serde_json::json!({
            "three": format!("{base}three.module.min.js"),
            "three/addons/": format!("{base}addons/"),
            "rama-x-threejs": format!("{base}rama-x-threejs.js"),
            "datastar": format!("{base}datastar.js"),
        })
    );
}

#[test]
fn the_page_can_keep_its_own_datastar_and_add_entries() {
    let map = ImportMap::new("/three/")
        .with_datastar("/assets/datastar.js")
        .with_import("scene/", "/static/scene/");
    let entries = imports(&map);
    assert_eq!(entries["datastar"], "/assets/datastar.js");
    assert_eq!(entries["scene/"], "/static/scene/");
    assert_eq!(
        entries["three"],
        format!("{}three.module.min.js", base("/three"))
    );

    let mut map = ImportMap::new("/three");
    map.set_datastar("/d.js").set_import("a", "/a.js");
    assert_eq!(imports(&map)["a"], "/a.js");
}

#[test]
fn renders_the_same_as_html_and_as_text() {
    let map = ImportMap::new("/three");
    assert_eq!((&map).into_string(), map.to_string());
    assert_eq!(map.clone().into_string(), map.to_string());
}

#[test]
fn is_safe_inside_a_script_element() {
    let html = ImportMap::new("/</script><script>alert(1)//").to_string();
    assert_eq!(html.matches("</script>").count(), 1, "{html}");
    assert!(html.contains(r"\u003c/script>"), "{html}");
}
