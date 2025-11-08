import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import MonacoEditor from './MonacoEditor';
import { readFile, writeFile, deleteFile } from '../bridge/files';

import { editorStore } from '@/layout/stores/EditorStore';

function CodeEditorViewport({ 
  tab = null
}) {
  const [editorValue, setEditorValue] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [, setSaving] = createSignal(false);
  const [error, setError] = createSignal(null);
  const [hasChanges, setHasChanges] = createSignal(false);
  const [originalValue, setOriginalValue] = createSignal('');
  const [fileName, setFileName] = createSignal('untitled.js');
  const [originalFileName, setOriginalFileName] = createSignal('');
  const [selectedFile, setSelectedFile] = createSignal(tab?.initialFile || null);
  const [parsedScript, setParsedScript] = createSignal(null);
  const [, setParseError] = createSignal(null);
  const [parseErrors, setParseErrors] = createSignal([]); // Array of errors with line numbers
  const [, setPreviousProperties] = createSignal([]);
  const [, setPreviousScriptContent] = createSignal(''); // Track full script content

  const getLanguageFromExtension = (fileName) => {
    const ext = fileName.toLowerCase().split('.').pop();
    const languageMap = {
      'js': 'javascript',
      'jsx': 'javascript',
      'ts': 'typescript',
      'tsx': 'typescript',
      'json': 'json',
      'html': 'html',
      'css': 'css',
      'md': 'markdown',
      'txt': 'plaintext',
      'xml': 'xml',
      'yaml': 'yaml',
      'yml': 'yaml'
    };
    return languageMap[ext] || 'plaintext';
  };


  const loadFile = async (file) => {
    if (!file) {
      // No file selected - start with empty editor
      setFileName('untitled.js');
      setOriginalFileName('');
      setEditorValue('');
      setOriginalValue('');
      setHasChanges(false);
      setParsedScript(null);
      setPreviousProperties([]);
      setParseError(null);
      setParseErrors([]);
      setPreviousScriptContent('');
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      // Project functionality disabled - cannot load files
      throw new Error('File loading is currently unavailable - project API removed');
      const content = await readFile(filePath);
      
      setFileName(file.name);
      setOriginalFileName(file.name);
      setEditorValue(content);
      setOriginalValue(content);
      setHasChanges(false);
    } catch (err) {
      setError(`Failed to load file: ${err.message}`);
      setEditorValue('');
      setOriginalValue('');
    } finally {
      setLoading(false);
    }
  };

  const saveFile = async () => {
    // File saving functionality disabled - project API removed
    setError('File saving is currently unavailable - project API removed');
    setSaving(false);
    return;

    if (!fileName().trim()) {
      setError('File name cannot be empty');
      return;
    }

    setSaving(true);
    setError(null);
    
    try {
      // Determine the file path in the current directory
      const newFilePath = `projects/${currentProject.name}/${fileName()}`;
      
      // If this is a rename (original file exists and name changed)
      if (originalFileName() && originalFileName() !== fileName() && selectedFile()) {
        // Delete the old file
        const oldFilePath = `projects/${currentProject.name}/${selectedFile().path}`;
        try {
          await deleteFile(oldFilePath);
        } catch (deleteErr) {
          // Continue anyway - the old file might not exist
        }
      }
      
      // Write the new/updated file
      await writeFile(newFilePath, editorValue());
      
      setOriginalValue(editorValue());
      setOriginalFileName(fileName());
      setHasChanges(false);
      
      // Dispatch event to trigger file change detection and refresh the asset list
      document.dispatchEvent(new CustomEvent('engine:file-saved', {
        detail: { path: newFilePath, content: editorValue() }
      }));
      
    } catch (err) {
      setError(`Failed to save file: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  // Debounce timeout for script parsing
  let parseTimeout = null;
  
  // Cleanup timeout on component unmount
  onCleanup(() => {
    if (parseTimeout) {
      clearTimeout(parseTimeout);
    }
  });
  
  const handleEditorChange = (value) => {
    setEditorValue(value);
    updateHasChanges(value, fileName());
  };


  const updateHasChanges = (content, name) => {
    const contentChanged = content !== originalValue();
    const nameChanged = name !== originalFileName();
    setHasChanges(contentChanged || nameChanged);
  };


  const handleKeyDown = (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      saveFile();
    }
  };

  // Load file when selectedFile changes or when editor opens
  createEffect(() => {
    const file = selectedFile();
    loadFile(file);
  });

  // Add keyboard event listener
  createEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  });

  // Listen for external file changes
  createEffect(() => {
    const handleExternalFileChange = (event) => {
      const { file } = event.detail;
      if (file) {
        setSelectedFile(file);
      }
    };

    document.addEventListener('engine:open-code-editor', handleExternalFileChange);
    return () => document.removeEventListener('engine:open-code-editor', handleExternalFileChange);
  });

  // Initialize with tab data
  createEffect(() => {
    if (tab?.initialFile) {
      setSelectedFile(tab.initialFile);
    }
  });

  // Ensure Monaco Editor responds to layout changes
  createEffect(() => {
    // Add a small delay to ensure layout has settled
    const timer = setTimeout(() => {
      // Trigger Monaco layout update when the viewport size changes
      window.dispatchEvent(new Event('resize'));
    }, 100);
    
    return () => clearTimeout(timer);
  });

  return (
    <div class="h-full w-full flex flex-col bg-base-100 relative">

        {/* Content */}
        <div class="flex-1 flex flex-col min-h-0">
          <Show when={error()}>
            <div class="p-3 bg-error/10 border-b border-error/20">
              <div class="text-sm text-error">{error()}</div>
            </div>
          </Show>

          <Show when={loading()}>
            <div class="flex-1 flex items-center justify-center">
              <div class="flex items-center gap-2 text-base-content/60">
                <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                <span class="text-sm">Loading file...</span>
              </div>
            </div>
          </Show>

          <Show when={!loading()}>
            <div class="flex-1 flex flex-col min-h-0">
              <div class="flex-1 min-h-0 relative">
                <MonacoEditor
                  value={editorValue()}
                  onChange={handleEditorChange}
                  language={getLanguageFromExtension(fileName())}
                  theme="vs-dark"
                  height="100%"
                  width="100%"
                  options={{
                    fontSize: 13,
                    lineHeight: 18,
                    minimap: { enabled: false },
                    scrollBeyondLastLine: false,
                    wordWrap: 'on',
                    automaticLayout: true,
                    folding: true,
                    renderWhitespace: 'boundary',
                    tabSize: 2,
                    insertSpaces: true
                  }}
                />
              </div>
              
            </div>
          </Show>
        </div>

        {/* Status Bar */}
        <div class="px-3 py-2 bg-base-200 border-t border-base-300 text-xs text-base-content/60">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span>{getLanguageFromExtension(fileName()).toUpperCase()}</span>
              <Show when={selectedFile()?.size}>
                <span>{Math.round(selectedFile().size / 1024)}KB</span>
              </Show>
            </div>
            <Show when={hasChanges()}>
              <span class="text-warning">• Modified</span>
            </Show>
          </div>
        </div>
      </div>
  );
}

export default CodeEditorViewport;