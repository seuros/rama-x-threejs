//! The `<script type="importmap">` that names the mounted files.

use std::fmt;

use rama::http::protocols::html::{IntoHtml, PreEscaped};

use crate::assets::{ADDONS_PATH, DATASTAR_PATH, HOOK_PATH, MODULE_PATH, base};

/// The page's import map, for files mounted with
/// [`with_three`](crate::ThreeRouterExt::with_three) under the same prefix.
///
/// It names `three`, `three/addons/`, `rama-x-threejs` (the hook) and
/// `datastar`, so a page writes `import * as THREE from 'three'`. Renders as
/// HTML ([`IntoHtml`]) or text ([`Display`](fmt::Display)); put it before the
/// first module script.
///
/// ```rust
/// use rama_x_threejs::ImportMap;
///
/// let head = ImportMap::new("/three").with_import("scene/", "/static/scene/").to_string();
/// assert!(head.starts_with(r#"<script type="importmap">{"imports":{"#));
/// ```
#[derive(Debug, Clone)]
pub struct ImportMap {
    prefix: String,
    datastar: Option<String>,
    imports: Vec<(String, String)>,
}

impl ImportMap {
    /// The import map for files mounted under `prefix`.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            datastar: None,
            imports: Vec::new(),
        }
    }

    /// Point `datastar` at the page's own copy instead of the mounted one.
    ///
    /// The hook imports `datastar`; a page must load Datastar from that one
    /// URL only, or it runs two Datastars.
    #[must_use]
    pub fn with_datastar(mut self, url: impl Into<String>) -> Self {
        self.set_datastar(url);
        self
    }

    /// See [`with_datastar`](Self::with_datastar).
    pub fn set_datastar(&mut self, url: impl Into<String>) -> &mut Self {
        self.datastar = Some(url.into());
        self
    }

    /// Add an entry of the app's, e.g. `("scene/", "/static/scene/")`.
    #[must_use]
    pub fn with_import(mut self, specifier: impl Into<String>, url: impl Into<String>) -> Self {
        self.set_import(specifier, url);
        self
    }

    /// See [`with_import`](Self::with_import).
    pub fn set_import(
        &mut self,
        specifier: impl Into<String>,
        url: impl Into<String>,
    ) -> &mut Self {
        self.imports.push((specifier.into(), url.into()));
        self
    }

    /// The JSON inside the element; `<` is escaped as `\u003c`.
    fn json(&self) -> String {
        let base = base(&self.prefix);
        let datastar = self
            .datastar
            .clone()
            .unwrap_or_else(|| format!("{base}{DATASTAR_PATH}"));
        let mut imports = serde_json::Map::new();
        for (specifier, url) in [
            ("three".to_owned(), format!("{base}{MODULE_PATH}")),
            ("three/addons/".to_owned(), format!("{base}{ADDONS_PATH}")),
            ("rama-x-threejs".to_owned(), format!("{base}{HOOK_PATH}")),
            ("datastar".to_owned(), datastar),
        ]
        .into_iter()
        .chain(self.imports.iter().cloned())
        {
            imports.insert(specifier, url.into());
        }
        serde_json::json!({ "imports": imports })
            .to_string()
            .replace('<', "\\u003c")
    }
}

impl fmt::Display for ImportMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"<script type="importmap">{}</script>"#, self.json())
    }
}

impl IntoHtml for ImportMap {
    fn into_html(self) -> impl IntoHtml {
        PreEscaped(self.to_string())
    }
}

impl IntoHtml for &ImportMap {
    fn into_html(self) -> impl IntoHtml {
        PreEscaped(self.to_string())
    }
}

#[cfg(test)]
mod tests;
