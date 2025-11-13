/**
 * Mock VS Code API for testing
 */

export enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2,
}

export enum QuickPickItemKind {
  Separator = -1,
  Default = 0,
}

export enum StatusBarAlignment {
  Left = 1,
  Right = 2,
}

export enum ConfigurationTarget {
  Global = 1,
  Workspace = 2,
  WorkspaceFolder = 3,
}

export class ThemeIcon {
  constructor(public id: string) {}
}

export class TreeItem {
  label?: string;
  description?: string;
  iconPath?: ThemeIcon;
  command?: any;
  tooltip?: string;
  contextValue?: string;
  collapsibleState?: TreeItemCollapsibleState;

  constructor(label: string, collapsibleState?: TreeItemCollapsibleState) {
    this.label = label;
    this.collapsibleState = collapsibleState;
  }
}

export class EventEmitter<T> {
  private listeners: Array<(e: T) => any> = [];

  get event() {
    return (listener: (e: T) => any) => {
      this.listeners.push(listener);
      return {
        dispose: () => {
          const index = this.listeners.indexOf(listener);
          if (index > -1) {
            this.listeners.splice(index, 1);
          }
        },
      };
    };
  }

  fire(data: T) {
    this.listeners.forEach((listener) => listener(data));
  }

  dispose() {
    this.listeners = [];
  }
}

export class CancellationTokenSource {
  token = {
    isCancellationRequested: false,
    onCancellationRequested: () => ({ dispose: () => {} }),
  };

  cancel() {
    this.token.isCancellationRequested = true;
  }

  dispose() {}
}

export const window = {
  showInformationMessage: async (message: string, ...items: string[]) =>
    items[0],
  showWarningMessage: async (message: string, ...items: string[]) => items[0],
  showErrorMessage: async (message: string, ...items: string[]) => items[0],
  showQuickPick: async (items: any[], options?: any) => items[0],
  showInputBox: async (options?: any) => 'test input',
  createStatusBarItem: (alignment?: StatusBarAlignment, priority?: number) => ({
    text: '',
    tooltip: '',
    command: undefined,
    show: () => {},
    hide: () => {},
    dispose: () => {},
  }),
  createOutputChannel: (name: string) => ({
    append: (value: string) => {},
    appendLine: (value: string) => {},
    clear: () => {},
    show: () => {},
    hide: () => {},
    dispose: () => {},
  }),
  createTreeView: (viewId: string, options: any) => ({
    reveal: () => {},
    dispose: () => {},
  }),
  registerTreeDataProvider: (viewId: string, provider: any) => ({
    dispose: () => {},
  }),
  activeTextEditor: undefined,
  showTextDocument: async (document: any) => ({}),
  visibleTextEditors: [],
};

export const workspace = {
  getConfiguration: (section?: string) => ({
    get: (key: string, defaultValue?: any) => defaultValue,
    has: (key: string) => false,
    inspect: (key: string) => undefined,
    update: async (key: string, value: any, target?: ConfigurationTarget) => {},
  }),
  workspaceFolders: [],
  onDidChangeConfiguration: () => ({ dispose: () => {} }),
  onDidChangeWorkspaceFolders: () => ({ dispose: () => {} }),
  fs: {
    readFile: async (uri: any) => Buffer.from(''),
    writeFile: async (uri: any, content: Uint8Array) => {},
    delete: async (uri: any) => {},
    createDirectory: async (uri: any) => {},
  },
  findFiles: async (pattern: string) => [],
  createFileSystemWatcher: (pattern: string) => ({
    onDidCreate: () => ({ dispose: () => {} }),
    onDidChange: () => ({ dispose: () => {} }),
    onDidDelete: () => ({ dispose: () => {} }),
    dispose: () => {},
  }),
};

export const commands = {
  registerCommand: (command: string, callback: (...args: any[]) => any) => ({
    dispose: () => {},
  }),
  executeCommand: async (command: string, ...args: any[]) => undefined,
};

export const languages = {
  registerCodeLensProvider: (selector: any, provider: any) => ({
    dispose: () => {},
  }),
  registerHoverProvider: (selector: any, provider: any) => ({
    dispose: () => {},
  }),
  registerCompletionItemProvider: (
    selector: any,
    provider: any,
    ...triggerCharacters: string[]
  ) => ({ dispose: () => {} }),
};

export class Uri {
  static file(path: string) {
    return {
      scheme: 'file',
      path,
      fsPath: path,
      toString: () => `file://${path}`,
    };
  }

  static parse(value: string) {
    return {
      scheme: 'file',
      path: value,
      fsPath: value,
      toString: () => value,
    };
  }
}

export class Range {
  constructor(
    public start: any,
    public end: any
  ) {}
}

export class Position {
  constructor(
    public line: number,
    public character: number
  ) {}
}

export class Selection extends Range {
  constructor(
    public anchor: Position,
    public active: Position
  ) {
    super(anchor, active);
  }
}

export class Disposable {
  static from(...disposables: { dispose: () => any }[]) {
    return {
      dispose: () => {
        disposables.forEach((d) => d.dispose());
      },
    };
  }
}

export const env = {
  openExternal: async (uri: Uri) => true,
};
