# rama-x-threejs

Serve a pinned three.js from rama, and drive your own scene code with Datastar signals.

## Install

```toml
[dependencies]
rama-x-threejs = { git = "https://github.com/seuros/rama-x-threejs" }
```

## Usage

```rust,ignore
use rama_x_threejs::{ImportMap, SceneSignals, ThreeRouterExt};

let router = Router::new()
    .with_three("/three") // three.js, its addons, the hook and Datastar
    .with_get("/", page)
    .with_get("/stream", stream)
    .with_get("/floor.js", Script(FLOOR_JS));

async fn page() -> Html<String> {
    Html(format!(
        r#"{}<script type="module">import "rama-x-threejs"</script>
<body data-init="@get('/stream')"><canvas data-three:floor="/floor.js"></canvas></body>"#,
        ImportMap::new("/three"),
    ))
}

// In the stream, for each new state:
SceneSignals::new("floor", &state).try_into_datastar_event()
```

```js
// floor.js: all of the scene is yours.
import * as THREE from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'

export function mount(canvas, state) { /* build the scene */ }
export function update(state) { /* apply the new state */ }
```

`cargo run --example spinner` runs it.

## three.js

0.186.1, with `OrbitControls`, `GLTFLoader`, `CSS2DRenderer`, `BufferGeometryUtils`
and `SkeletonUtils` under `three/addons/`. `mise run vendor` rebuilds
`assets/three/` from npm.

## License

MIT, see [LICENSE](LICENSE). three.js is MIT too; its notice is in
[assets/three/LICENSE](assets/three/LICENSE).
