// The data-three plugin against a stand-in for Datastar's exports: a plain
// root object, effects re-run by hand, and the plugin captured on register.
import { expect, mock, test } from 'bun:test'

const effects = new Set()
const datastar = {
  plugin: null,
  root: {},
  attribute(plugin) {
    datastar.plugin = plugin
  },
  effect(fn) {
    effects.add(fn)
    fn()
    return () => effects.delete(fn)
  },
}
mock.module('datastar', () => ({ attribute: datastar.attribute, effect: datastar.effect, root: datastar.root }))
globalThis.document = { baseURI: new URL('./', import.meta.url).href }
await import('../assets/rama-x-threejs.js')

const rerun = () => [...effects].forEach((fn) => fn())
const settle = () => new Promise((resolve) => setTimeout(resolve, 20))
const send = (name, state) => {
  datastar.root._three = { ...datastar.root._three, [name]: JSON.stringify(state) }
  rerun()
}
const quiet = async (fn) => {
  const error = console.error
  const logged = []
  console.error = (...args) => logged.push(args)
  try {
    await fn()
  } finally {
    console.error = error
  }
  return logged
}

test('registers data-three, needing a scene name and a module', () => {
  expect(datastar.plugin.name).toBe('three')
  expect(datastar.plugin.requirement).toEqual({ key: 'must', value: 'must' })
})

test('mounts with the current state, then updates on each new one', async () => {
  globalThis.calls = []
  send('floor', { belts: 1 })
  const el = { id: 'canvas' }
  const cleanup = datastar.plugin.apply({ el, key: 'floor', value: './fixtures/scene.js' })
  await settle()
  expect(calls).toEqual([['mount', el, { belts: 1 }]])

  send('floor', { belts: 2 })
  send('floor', { belts: 2 }) // the same text again: nothing new to draw
  send('other', { belts: 9 }) // another scene's signal
  expect(calls.slice(1)).toEqual([['update', { belts: 2 }, el]])

  cleanup()
  send('floor', { belts: 3 })
  expect(calls.slice(2)).toEqual([['unmount', el]])
})

test('mounts with no state before the server sent any, and updates when it does', async () => {
  globalThis.calls = []
  const el = { id: 'late' }
  datastar.plugin.apply({ el, key: 'late', value: './fixtures/scene.js' })
  await settle()
  expect(calls).toEqual([['mount', el, undefined]])
  send('late', [1])
  expect(calls.slice(1)).toEqual([['update', [1], el]])
})

test('skips bad JSON and survives a failing update', async () => {
  globalThis.calls = []
  globalThis.failUpdate = true
  const el = { id: 'bad' }
  datastar.plugin.apply({ el, key: 'bad', value: './fixtures/scene.js' })
  await settle()
  const logged = await quiet(() => {
    datastar.root._three = { ...datastar.root._three, bad: '{nope' }
    rerun()
    send('bad', { ok: true })
  })
  globalThis.failUpdate = false
  expect(logged.length).toBe(2)
  send('bad', { ok: 2 })
  expect(calls.at(-1)).toEqual(['update', { ok: 2 }, el])
})

test('an element removed while its module loads is never mounted', async () => {
  globalThis.calls = []
  const cleanup = datastar.plugin.apply({ el: {}, key: 'gone', value: './fixtures/scene.js' })
  cleanup()
  await settle()
  expect(calls).toEqual([])
})

test('reports a module that does not load', async () => {
  const logged = await quiet(async () => {
    datastar.plugin.apply({ el: {}, key: 'x', value: './fixtures/missing.js' })
    await settle()
  })
  expect(String(logged[0]?.[0])).toContain('could not load ./fixtures/missing.js')
})
