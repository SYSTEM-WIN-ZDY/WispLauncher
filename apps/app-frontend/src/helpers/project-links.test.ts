import assert from 'node:assert/strict'
import test from 'node:test'

import { createProjectBrowseLocation } from './project-links.ts'

test('project sidebar loader links target the app browse route and loader filter', () => {
	assert.deepEqual(createProjectBrowseLocation('mod', 'loader', 'forge'), {
		path: '/browse/mod',
		query: { g: 'categories:forge' },
	})
})

test('project sidebar category links target the app browse route and category filter', () => {
	assert.deepEqual(createProjectBrowseLocation('mod', 'category', 'library-api'), {
		path: '/browse/mod',
		query: { f: 'categories:library-api' },
	})
})

test('server categories use the server browse route and server category filter', () => {
	assert.deepEqual(createProjectBrowseLocation('minecraft_java_server', 'category', 'vanilla'), {
		path: '/browse/server',
		query: { sc: 'vanilla' },
	})
})
