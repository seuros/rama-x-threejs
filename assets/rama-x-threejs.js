// rama-x-threejs: the `data-three` Datastar attribute.
//
//   <canvas data-three:floor="/game/floor.js"></canvas>
//
// imports /game/floor.js, calls its `mount(el, state)` once, then its
// `update(state, el)` whenever the server sends
// `SceneSignals::new("floor", &state)`. `state` arrives decoded; every line of
// three.js stays in the app's module. `mount` may return a cleanup function,
// called when the element leaves the page.
//
// The value is a module specifier: a path resolves against the page, a bare
// name goes through the import map.
import { attribute, effect, root } from 'datastar'

// SceneSignals sets `_three.<name>` to the state as JSON text.
const NAMESPACE = '_three'

/** The decoded state of scene `name` and its JSON text, or undefined. */
function read(name) {
  // Through the root proxy, not getPath: inside an effect this subscribes
  // even to a signal the server has not sent yet.
  const text = root[NAMESPACE]?.[name]
  if (typeof text !== 'string' || text === '') return undefined
  try {
    return { text, state: JSON.parse(text) }
  } catch (error) {
    console.error(`rama-x-threejs: scene "${name}" is not JSON`, error)
    return undefined
  }
}

function specifier(value) {
  return /^(\.{0,2}\/|[a-z][a-z0-9+.-]*:)/i.test(value) ? new URL(value, document.baseURI).href : value
}

attribute({
  name: 'three',
  requirement: { key: 'must', value: 'must' },
  apply({ el, key, value }) {
    let removed = false
    let stop
    let unmount
    import(specifier(value))
      .then(async (scene) => {
        if (removed) return
        const first = read(key)
        unmount = await scene.mount?.(el, first?.state)
        if (removed) {
          if (typeof unmount === 'function') unmount()
          return
        }
        let shown = first?.text
        stop = effect(() => {
          const next = read(key)
          if (!next || next.text === shown) return
          shown = next.text
          try {
            scene.update?.(next.state, el)
          } catch (error) {
            console.error(`rama-x-threejs: update of scene "${key}" failed`, error)
          }
        })
      })
      .catch((error) => console.error(`rama-x-threejs: data-three:${key} could not load ${value}`, error))
    return () => {
      removed = true
      stop?.()
      if (typeof unmount === 'function') unmount()
    }
  },
})
