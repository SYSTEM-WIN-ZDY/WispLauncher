import type { Component } from 'vue'

export type AboutMemberExperience = {
	component: Component
	longPressDuration: number
}

export function getAboutMemberExperience(_experience: unknown): AboutMemberExperience | undefined {
	return undefined
}
