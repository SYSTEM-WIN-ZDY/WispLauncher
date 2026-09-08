import {
	buildFileTreeRows,
	collectFileTreeFolders,
	type FileTreeFileRow,
	type FileTreeFolderRow,
	type FileTreeRow,
} from '@modrinth/ui/src/utils/file-tree.ts'

import type { InstanceSchematicFile } from './backend'

export type InstanceSchematicFolderRow = FileTreeFolderRow
export type InstanceSchematicFileRow = FileTreeFileRow<InstanceSchematicFile>
export type InstanceSchematicRow = FileTreeRow<InstanceSchematicFile>

export function collectSchematicFolders(files: readonly InstanceSchematicFile[]): string[] {
	return collectFileTreeFolders(files)
}

export function buildInstanceSchematicRows(
	files: readonly InstanceSchematicFile[],
	expandedFolders: ReadonlySet<string>,
	searchQuery: string,
	locale = 'en',
): InstanceSchematicRow[] {
	return buildFileTreeRows(files, expandedFolders, searchQuery, locale)
}
