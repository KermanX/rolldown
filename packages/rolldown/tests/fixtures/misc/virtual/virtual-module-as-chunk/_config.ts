import { defineTest } from '@tests'
import { expect, vi } from 'vitest'

const fn = vi.fn()
import { getOutputChunkNames } from '@tests/utils'

export default defineTest({
  config: {
    input: ['main.js', 'entry.js'],
    plugins: [
      {
        resolveId(id) {
          if (id === '\0module') {
            return id
          }
        },
        load(id) {
          if (id === '\0module') {
            fn()
            return `export default 'module'`
          }
        },
      },
    ],
  },
  afterTest(output) {
    // cSpell:disable
    expect(getOutputChunkNames(output)).toMatchInlineSnapshot(`
      [
        "_module-KtCeJTRH.js",
        "entry.js",
        "main.js",
      ]
    `)
  },
})
