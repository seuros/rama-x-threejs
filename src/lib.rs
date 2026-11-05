// SPDX-License-Identifier: MIT
// Copyright (c) 2026, Abdelkader Boudih
// See the LICENSE file at the root of this crate for the full license text.

//! Serve a pinned [three.js] from [rama] and drive the page's own scene code
//! with [Datastar] signals.
//!
//! - [`ThreeModule`], [`ThreeAddon`], [`ThreeHook`]: the vendored files as
//!   [`IntoResponse`](rama::http::service::web::response::IntoResponse)
//!   types, like rama's `DatastarScript`.
//! - [`ThreeRouterExt::with_three`]: mounts them, and Datastar, in one line.
//! - [`ImportMap`]: the `<script type="importmap">` that names them.
//! - [`SceneSignals`]: a [`PatchSignals`](rama::http::sse::datastar::PatchSignals)
//!   carrying one scene's state.
//! - The hook adds one Datastar attribute, `data-three:<name>="<module>"`: it
//!   imports the module and calls its `mount(el, state)`, then its
//!   `update(state, el)` for each new state. The module holds all the
//!   three.js code.
//!
//! ```rust
//! use rama::http::service::web::Router;
//! use rama_x_threejs::{ImportMap, SceneSignals, ThreeRouterExt};
//!
//! let router: Router = Router::new().with_three("/three");
//!
//! let page = format!(
//!     r#"{}<script type="module">import "rama-x-threejs"</script>
//! <body data-init="@get('/stream')"><canvas data-three:floor="/floor.js"></canvas></body>"#,
//!     ImportMap::new("/three"),
//! );
//!
//! // In the /stream handler, for each new state:
//! let event = SceneSignals::new("floor", &[1, 2, 3]).try_into_datastar_event()?;
//! # drop((router, page, event));
//! # Ok::<(), rama_x_threejs::SceneSignalsError>(())
//! ```
//!
//! [three.js]: https://threejs.org
//! [rama]: https://ramaproxy.org
//! [Datastar]: https://data-star.dev

mod assets;
mod import_map;
mod scene_signals;

pub use assets::{ThreeAddon, ThreeHook, ThreeModule, ThreeRouterExt};
pub use import_map::ImportMap;
pub use scene_signals::{SceneSignals, SceneSignalsError};

/// The vendored three.js release, the npm `three` package version.
pub const THREE_VERSION: &str = "0.186.1";
