//! Scene state as a Datastar signal patch.

use std::fmt;

use rama::http::sse::datastar::{DatastarEvent, PatchSignals};
use serde::Serialize;

/// The signal every scene's state lives under: `_three.<name>`. The `_` keeps
/// Datastar from sending it back with the page's own requests.
const NAMESPACE: &str = "_three";

/// A [`PatchSignals`] that hands `state` to the page's `data-three:<name>`
/// scene.
///
/// The state travels as JSON text in one string signal: Datastar replaces a
/// string whole, where it would merge an object key by key (so a dropped key
/// would linger) and make every nested field reactive. Datastar fires nothing
/// for a value it already holds, so an identical state reaches no `update`.
///
/// ```rust
/// use rama_x_threejs::SceneSignals;
///
/// let event = SceneSignals::new("floor", &[1, 2, 3]).try_into_datastar_event()?;
/// # drop(event);
/// # Ok::<(), rama_x_threejs::SceneSignalsError>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SceneSignals<'a, T: ?Sized> {
    name: &'a str,
    state: &'a T,
    only_if_missing: bool,
}

impl<'a, T: Serialize + ?Sized> SceneSignals<'a, T> {
    /// The state of scene `name`: lowercase ASCII letters, digits and `_`,
    /// starting with a letter, as in the `data-three:<name>` attribute (HTML
    /// lowercases attribute names).
    pub const fn new(name: &'a str, state: &'a T) -> Self {
        Self {
            name,
            state,
            only_if_missing: false,
        }
    }

    /// Set the state only if the page holds none yet, as
    /// [`PatchSignals::with_only_if_missing`].
    #[must_use]
    pub const fn with_only_if_missing(mut self, only_if_missing: bool) -> Self {
        self.only_if_missing = only_if_missing;
        self
    }

    /// See [`with_only_if_missing`](Self::with_only_if_missing).
    pub const fn set_only_if_missing(&mut self, only_if_missing: bool) -> &mut Self {
        self.only_if_missing = only_if_missing;
        self
    }

    /// The [`PatchSignals`] that sets `_three.<name>` to the state's JSON.
    pub fn try_into_patch_signals(self) -> Result<PatchSignals, SceneSignalsError> {
        if !valid_name(self.name) {
            return Err(SceneSignalsError(Kind::Name(self.name.to_owned())));
        }
        let text =
            serde_json::to_string(self.state).map_err(|e| SceneSignalsError(Kind::Json(e)))?;
        let signals = serde_json::json!({ NAMESPACE: { self.name: text } }).to_string();
        Ok(PatchSignals::new(signals).with_only_if_missing(self.only_if_missing))
    }

    /// The patch as a Datastar SSE event.
    pub fn try_into_datastar_event(self) -> Result<DatastarEvent, SceneSignalsError> {
        self.try_into_patch_signals()?
            .try_into_datastar_event()
            .map_err(|e| SceneSignalsError(Kind::Event(e.to_string())))
    }
}

impl<T: Serialize + ?Sized> TryFrom<SceneSignals<'_, T>> for DatastarEvent {
    type Error = SceneSignalsError;

    fn try_from(value: SceneSignals<'_, T>) -> Result<Self, Self::Error> {
        value.try_into_datastar_event()
    }
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Why a [`SceneSignals`] could not become a patch.
#[derive(Debug)]
pub struct SceneSignalsError(Kind);

#[derive(Debug)]
enum Kind {
    Name(String),
    Json(serde_json::Error),
    Event(String),
}

impl fmt::Display for SceneSignalsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Kind::Name(name) => write!(
                f,
                "scene name {name:?} must be lowercase ASCII letters, digits and _, starting with a letter"
            ),
            Kind::Json(e) => write!(f, "scene state did not serialize: {e}"),
            Kind::Event(e) => write!(f, "scene event did not build: {e}"),
        }
    }
}

impl std::error::Error for SceneSignalsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            Kind::Json(e) => Some(e),
            Kind::Name(_) | Kind::Event(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
