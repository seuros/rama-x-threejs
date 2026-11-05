// A scene module as an app writes one, recording what the hook calls.
export function mount(el, state) {
  globalThis.calls.push(['mount', el, state])
  return () => globalThis.calls.push(['unmount', el])
}

export function update(state, el) {
  if (globalThis.failUpdate) throw new Error('update failed')
  globalThis.calls.push(['update', state, el])
}
