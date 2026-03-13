import { test as base, type Page } from '@playwright/test';

/** Sample template data returned by the `list_templates` Tauri command. */
export const MOCK_TEMPLATES = [
  {
    name: 'loan-eligibility',
    description: 'Chained decisions with UNIQUE hit policy for loan approval',
    nodeCount: 3,
    features: ['UNIQUE hit policy', 'Chained decisions', 'Risk rating'],
  },
  {
    name: 'insurance-pricing',
    description: 'BKM with age-based pricing, FEEL ranges, and literal expressions',
    nodeCount: 3,
    features: ['BKM nodes', 'FEEL ranges', 'Literal expressions'],
  },
  {
    name: 'tax-calculator',
    description: 'Progressive tax brackets with numeric ranges and chained calculation',
    nodeCount: 3,
    features: ['Numeric ranges', 'Progressive brackets', 'COLLECT policy'],
  },
  {
    name: 'order-routing',
    description: 'Multi-input decision tables for logistics branching',
    nodeCount: 3,
    features: ['Multi-input tables', 'Logistics branching', 'FIRST policy'],
  },
  {
    name: 'compliance-checker',
    description: 'Knowledge sources, governed-by relationships, and boolean logic',
    nodeCount: 4,
    features: ['Knowledge sources', 'Governed-by edges', 'Boolean logic'],
  },
  {
    name: 'scorecard',
    description: 'Weighted BKM scoring with chained decision contexts',
    nodeCount: 5,
    features: ['BKM scoring', 'Weighted factors', 'Chained contexts'],
  },
];

/** Sample recent project entries. */
export const MOCK_RECENT_PROJECTS = [
  {
    path: '/home/user/projects/loan-app',
    name: 'loan-app',
    lastOpened: new Date().toISOString(),
  },
  {
    path: '/home/user/projects/insurance-demo',
    name: 'insurance-demo',
    lastOpened: new Date(Date.now() - 86400000).toISOString(),
  },
  {
    path: '/home/user/projects/old-project',
    name: 'old-project',
    lastOpened: new Date(Date.now() - 86400000 * 10).toISOString(),
  },
];

/** Sample flow graph returned by `load_dmn_project`. */
export const MOCK_FLOW_GRAPH = {
  nodes: [
    {
      id: 'applicant-input',
      label: 'Applicant Input',
      nodeType: 'inputData',
      dataTypeRef: 'ApplicantData',
      schemaPath: '../types/loan.ts',
      body: 'Collects applicant information.',
      sourceFile: 'decisions/applicant_input.md',
    },
    {
      id: 'risk-rating',
      label: 'Risk Rating',
      nodeType: 'decision',
      dataTypeRef: 'RiskLevel',
      schemaPath: null,
      body: 'Determines risk based on credit score.',
      sourceFile: 'decisions/risk_rating.md',
    },
    {
      id: 'loan-eligibility',
      label: 'Loan Eligibility',
      nodeType: 'decision',
      dataTypeRef: 'EligibilityResult',
      schemaPath: null,
      body: 'Final eligibility decision.',
      sourceFile: 'decisions/loan_eligibility.md',
    },
  ],
  edges: [
    { id: 'e0', source: 'applicant-input', target: 'risk-rating', edgeType: 'requires' },
    { id: 'e1', source: 'risk-rating', target: 'loan-eligibility', edgeType: 'requires' },
  ],
  projectName: 'loan-app',
  typeFiles: [
    {
      path: 'types/loan.ts',
      content: 'export interface ApplicantData {\n  name: string;\n  creditScore: number;\n}',
    },
  ],
};

/** Flow graph with no nodes — avoids SvelteFlow lifecycle errors in tests. */
export const MOCK_FLOW_GRAPH_EMPTY = {
  ...MOCK_FLOW_GRAPH,
  nodes: [],
  edges: [],
};

/** Sample decision table info. */
export const MOCK_DECISION_TABLE = {
  nodeId: 'risk-rating',
  hitPolicy: 'UNIQUE',
  inputColumns: ['Credit Score'],
  outputColumns: ['Risk Level'],
  rules: [
    { index: 0, inputEntries: ['< 500'], outputEntries: ['"High"'] },
    { index: 1, inputEntries: ['[500..700)'], outputEntries: ['"Medium"'] },
    { index: 2, inputEntries: ['>= 700'], outputEntries: ['"Low"'] },
  ],
};

/**
 * Injects the Tauri mock into the page so the bridge thinks it's running in Tauri.
 *
 * The `@tauri-apps/api/core` invoke function delegates to
 * `window.__TAURI_INTERNALS__.invoke`, and the dialog plugin sends
 * `plugin:dialog|open` through the same path.
 */
export async function injectTauriMock(
  page: Page,
  opts: {
    recentProjects?: typeof MOCK_RECENT_PROJECTS;
    templates?: typeof MOCK_TEMPLATES;
    flowGraph?: typeof MOCK_FLOW_GRAPH;
    decisionTable?: typeof MOCK_DECISION_TABLE;
    /** Value returned by the directory dialog (null = user cancelled). */
    dialogResult?: string | null;
  } = {}
) {
  const recentProjects = opts.recentProjects ?? [];
  const templates = opts.templates ?? MOCK_TEMPLATES;
  const flowGraph = opts.flowGraph ?? MOCK_FLOW_GRAPH;
  const decisionTable = opts.decisionTable ?? MOCK_DECISION_TABLE;
  const dialogResult = opts.dialogResult ?? null;

  await page.addInitScript(
    ({ recentProjects, templates, flowGraph, decisionTable, dialogResult }) => {
      const _recentProjects = [...recentProjects];

      // Track all invoke calls for test assertions
      (window as any).__invokeLog = [] as Array<{ cmd: string; args: any }>;

      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
          (window as any).__invokeLog.push({ cmd, args });

          switch (cmd) {
            case 'list_templates':
              return templates;
            case 'get_recent_projects':
              return [..._recentProjects];
            case 'add_recent_project': {
              const idx = _recentProjects.findIndex((p: any) => p.path === args.path);
              if (idx >= 0) _recentProjects.splice(idx, 1);
              _recentProjects.unshift({
                path: args.path,
                name: args.name,
                lastOpened: new Date().toISOString(),
              });
              if (_recentProjects.length > 20) _recentProjects.length = 20;
              return;
            }
            case 'remove_recent_project': {
              const ri = _recentProjects.findIndex((p: any) => p.path === args.path);
              if (ri >= 0) _recentProjects.splice(ri, 1);
              return;
            }
            case 'load_dmn_project':
              return flowGraph;
            case 'parse_decision_table':
              return decisionTable;
            case 'create_project_from_template':
              return args.destDir ?? args.dest_dir;
            case 'evaluate_feel_expression':
              return '42';
            // Dialog plugin — open directory picker
            case 'plugin:dialog|open':
              return dialogResult;
            default:
              throw new Error(`Unknown Tauri command: ${cmd}`);
          }
        },
        metadata: () => Promise.resolve({}),
        convertFileSrc: (src: string) => src,
      };
    },
    { recentProjects, templates, flowGraph, decisionTable, dialogResult }
  );
}

/**
 * Extended test fixture with pre-configured pages.
 */
export const test = base.extend<{
  /** Page with no recent projects → shows WelcomeScreen. */
  welcomePage: Page;
  /** Page with recent projects → shows ProjectPicker. */
  pickerPage: Page;
}>({
  welcomePage: async ({ page }, use) => {
    await injectTauriMock(page, { recentProjects: [], templates: MOCK_TEMPLATES });
    await page.goto('/');
    await page.waitForSelector('.welcome', { timeout: 5000 });
    await use(page);
  },
  pickerPage: async ({ page }, use) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });
    await use(page);
  },
});

export { expect } from '@playwright/test';
