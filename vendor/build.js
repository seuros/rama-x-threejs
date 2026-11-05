// Rebuilds ../assets/three/ from the three npm package pinned in package.json.
//
// three stopped publishing minified builds with r186, so build/three.module.js
// (which imports three.core.js) is bundled and minified here into one file.
// The addons are copied verbatim. CI reruns this and fails if the committed
// files differ, so the vendored bytes are always reproducible from npm.
import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'

const pkg = join(import.meta.dir, 'node_modules/three')
const out = join(import.meta.dir, '../assets/three')

// GLTFLoader imports the two utils by relative path, so the npm layout under
// examples/jsm/ is kept as is.
const ADDONS = [
  'controls/OrbitControls.js',
  'loaders/GLTFLoader.js',
  'renderers/CSS2DRenderer.js',
  'utils/BufferGeometryUtils.js',
  'utils/SkeletonUtils.js',
]

const { version } = JSON.parse(await readFile(join(pkg, 'package.json'), 'utf8'))

await rm(out, { recursive: true, force: true })
await mkdir(join(out, 'build'), { recursive: true })

const entry = join(pkg, 'build/three.module.js')
const built = await Bun.build({ entrypoints: [entry], minify: true, format: 'esm', target: 'browser' })
if (!built.success) {
  for (const log of built.logs) console.error(log)
  process.exit(1)
}
// The minifier drops the @license header; put three's own back on top.
const banner = (await readFile(entry, 'utf8')).match(/^\/\*\*[\s\S]*?\*\/\n/)?.[0]
if (!banner?.includes('@license')) throw new Error('three.module.js no longer starts with its @license header')
await writeFile(join(out, 'build/three.module.min.js'), banner + (await built.outputs[0].text()))

for (const addon of ADDONS) {
  const target = join(out, 'examples/jsm', addon)
  await mkdir(dirname(target), { recursive: true })
  await cp(join(pkg, 'examples/jsm', addon), target)
}
await cp(join(pkg, 'LICENSE'), join(out, 'LICENSE'))
await writeFile(join(out, 'VERSION'), `${version}\n`)
console.log(`vendored three ${version}`)
