import { isServer } from '@tanstack/react-query';

if (!isServer && typeof window !== 'undefined') {
  window.MonacoEnvironment = {
    getWorker(moduleId, label) {
      switch (label) {
        case 'editorWorkerService':
          return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker', import.meta.url));
        case 'css':
        case 'less':
        case 'scss':
          return new Worker(
            new URL('monaco-editor/esm/vs/language/css/css.worker', import.meta.url)
          );
        case 'handlebars':
        case 'html':
        case 'razor':
          return new Worker(
            new URL('monaco-editor/esm/vs/language/html/html.worker', import.meta.url)
          );
        case 'json':
          return new Worker(
            new URL('monaco-editor/esm/vs/language/json/json.worker', import.meta.url)
          );
        case 'javascript':
        case 'typescript':
          return new Worker(
            new URL('monaco-editor/esm/vs/language/typescript/ts.worker', import.meta.url)
          );
        case 'yaml':
          return new Worker(new URL('monaco-yaml/yaml.worker', import.meta.url));
        default:
          throw new Error(`Unknown label ${label}`);
      }
    }
  };
}
