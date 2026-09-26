import { defineConfig } from 'seasoned-skills'

export default defineConfig({
  projectName: 'mundareu',
  contentDir: 'workflow-content',
  mergeStrategy: 'merge-commit',
  // Agents never merge to the base branch during a goal unless the project opts in.
  agentMergesDuringGoal: false,
  outOfScopeFindings: 'bank',
  release: { target: 'deployed-product' },
  gates: {
    lint: 'pnpm check',
    typecheck: 'pnpm tsc',
    full: ['pnpm check', 'pnpm tsc'],
  },
  calibrationFile: 'workflow-content/calibrations.md',
  // The resource table isolated worktree lanes are provisioned from. Uncomment
  // and describe what each repository owns; without it, a lane is a worktree and
  // nothing else. `provision <lane> --repo <path>` picks which entries a lane covers.
  // provisioning: {
  //   repositories: [
  //     {
  //       path: '.',
  //       migrateCommand: 'pnpm migrate',
  //       databases: [{ name: 'primary', seeded: true }],
  //       portBases: { web: 3000 },
  //     },
  //   ],
  // },
  // Whole criteria the project injects beyond the core, each backed by its own gate.
  additionalCriteria: [],
  // Quick-mode disqualifiers added to the package's base list.
  quickDisqualifiers: [],
})
