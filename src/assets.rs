//! The vendored files as responses, and the router extension that mounts them.

use std::sync::LazyLock;

use rama::http::header;
use rama::http::service::web::Router;
use rama::http::service::web::response::{DatastarScript, DatastarSourceMap, IntoResponse};
use rama::http::{Response, StatusCode};

/// A year, the longest max-age caches honour: the path carries the content hash.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

const MODULE: &[u8] = include_bytes!("../assets/three/build/three.module.min.js");
const HOOK: &[u8] = include_bytes!("../assets/rama-x-threejs.js");

/// Where each file sits under `{prefix}/{bundle}/`.
pub(crate) const MODULE_PATH: &str = "three.module.min.js";
pub(crate) const ADDONS_PATH: &str = "addons/";
pub(crate) const HOOK_PATH: &str = "rama-x-threejs.js";
pub(crate) const DATASTAR_PATH: &str = "datastar.js";

/// three.js itself, minified into one module (three's core included).
///
/// Served with `Cache-Control: public, max-age=31536000, immutable`: mount it
/// under a path that changes with the content, as
/// [`with_three`](ThreeRouterExt::with_three) does.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ThreeModule;

impl ThreeModule {
    /// three.js, ready to be served.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IntoResponse for ThreeModule {
    fn into_response(self) -> Response {
        javascript(MODULE)
    }
}

/// The `data-three` Datastar attribute: it imports the app's scene module and
/// feeds it [`SceneSignals`](crate::SceneSignals).
///
/// It imports `datastar` through the [`ImportMap`](crate::ImportMap). Cached
/// like [`ThreeModule`].
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ThreeHook;

impl ThreeHook {
    /// The hook, ready to be served.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IntoResponse for ThreeHook {
    fn into_response(self) -> Response {
        javascript(HOOK)
    }
}

/// A vendored three.js addon, imported as `three/addons/{path}`.
///
/// GLTFLoader imports both utils by relative path, so serve them side by side
/// under the same directory, as [`with_three`](ThreeRouterExt::with_three)
/// does. Cached like [`ThreeModule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThreeAddon {
    /// `controls/OrbitControls.js`
    OrbitControls,
    /// `loaders/GLTFLoader.js`
    GltfLoader,
    /// `renderers/CSS2DRenderer.js`
    Css2dRenderer,
    /// `utils/BufferGeometryUtils.js`, needed by GLTFLoader.
    BufferGeometryUtils,
    /// `utils/SkeletonUtils.js`, needed by GLTFLoader.
    SkeletonUtils,
}

impl ThreeAddon {
    /// Every vendored addon.
    pub const ALL: [Self; 5] = [
        Self::OrbitControls,
        Self::GltfLoader,
        Self::Css2dRenderer,
        Self::BufferGeometryUtils,
        Self::SkeletonUtils,
    ];

    /// Its path under `three/addons/`, e.g. `loaders/GLTFLoader.js`.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::OrbitControls => "controls/OrbitControls.js",
            Self::GltfLoader => "loaders/GLTFLoader.js",
            Self::Css2dRenderer => "renderers/CSS2DRenderer.js",
            Self::BufferGeometryUtils => "utils/BufferGeometryUtils.js",
            Self::SkeletonUtils => "utils/SkeletonUtils.js",
        }
    }

    const fn bytes(self) -> &'static [u8] {
        match self {
            Self::OrbitControls => {
                include_bytes!("../assets/three/examples/jsm/controls/OrbitControls.js")
            }
            Self::GltfLoader => {
                include_bytes!("../assets/three/examples/jsm/loaders/GLTFLoader.js")
            }
            Self::Css2dRenderer => {
                include_bytes!("../assets/three/examples/jsm/renderers/CSS2DRenderer.js")
            }
            Self::BufferGeometryUtils => {
                include_bytes!("../assets/three/examples/jsm/utils/BufferGeometryUtils.js")
            }
            Self::SkeletonUtils => {
                include_bytes!("../assets/three/examples/jsm/utils/SkeletonUtils.js")
            }
        }
    }
}

impl IntoResponse for ThreeAddon {
    fn into_response(self) -> Response {
        javascript(self.bytes())
    }
}

fn javascript(body: &'static [u8]) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, IMMUTABLE),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

/// A hash over every file this crate serves: the path segment that makes
/// their immutable caching safe across upgrades.
static BUNDLE: LazyLock<String> = LazyLock::new(|| {
    let files = [MODULE, HOOK]
        .into_iter()
        .chain(ThreeAddon::ALL.map(ThreeAddon::bytes));
    format!("{:016x}", files.fold(FNV_OFFSET, fnv1a))
});

/// `{prefix}/{bundle}/`, the directory every file is served under.
pub(crate) fn base(prefix: &str) -> String {
    format!("{}/{}/", prefix.trim_end_matches('/'), *BUNDLE)
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a, 64 bits: a cache key, not a defence.
fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Mount three.js on a rama [`Router`].
pub trait ThreeRouterExt {
    /// Serve, under `{prefix}/{bundle}/` where `bundle` hashes the content:
    ///
    /// | path                   | response               |
    /// |------------------------|------------------------|
    /// | `three.module.min.js`  | [`ThreeModule`]        |
    /// | `addons/{path}`        | every [`ThreeAddon`]   |
    /// | `rama-x-threejs.js`    | [`ThreeHook`]          |
    /// | `datastar.js`, `.map`  | rama's [`DatastarScript`] and [`DatastarSourceMap`] |
    ///
    /// [`ImportMap::new`](crate::ImportMap::new) with the same prefix names
    /// them for the page.
    #[must_use]
    fn with_three(self, prefix: impl AsRef<str>) -> Self;
}

impl<State> ThreeRouterExt for Router<State>
where
    State: Clone + Send + Sync + 'static,
{
    fn with_three(self, prefix: impl AsRef<str>) -> Self {
        let base = base(prefix.as_ref());
        let router = self
            .with_get(format!("{base}{MODULE_PATH}"), ThreeModule)
            .with_get(format!("{base}{HOOK_PATH}"), ThreeHook)
            .with_get(format!("{base}{DATASTAR_PATH}"), DatastarScript::new())
            .with_get(
                format!("{base}{DATASTAR_PATH}.map"),
                DatastarSourceMap::new(),
            );
        ThreeAddon::ALL.into_iter().fold(router, |router, addon| {
            router.with_get(format!("{base}{ADDONS_PATH}{}", addon.path()), addon)
        })
    }
}

#[cfg(test)]
mod tests;
