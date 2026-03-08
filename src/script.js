const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const fileList = document.getElementById('file-list');
const pathInput = document.getElementById('path');
let globalPath = "";
let selectedPath = "";
let showHidden = false;
let history = [];
let currentHistoryIndex = -1;

async function openFile(path) {
    try {
        await invoke('open_file', { path: path });
    } catch (e) {
        alert("Error opening file: " + e);
    }
}

async function goUp() {
    if (globalPath === "/") return;
    try {
        const parentPath = await invoke('get_parent_path', { path: globalPath });
        loadFiles(parentPath);
    } catch (e) {
        alert("Error going to parent directory: " + e);
    }
}

document.getElementById('up').onclick = async () => {
    await goUp();
}

async function loadFiles(path, addToHistory = true) {
    if (addToHistory) {
        history = history.slice(0, currentHistoryIndex + 1);
        history.push(path);
        currentHistoryIndex++;
    }
    globalPath = path;
    pathInput.value = path;
    try {
        const files = await invoke('get_files', { path: path });
        fileList.innerHTML = '';
        for (const file of files) {
            if (file.display_path.startsWith('.') && !showHidden) continue;
            const li = document.createElement('li');
            if (file.entry_type == "dir") {
                li.textContent = "📁 " + file.display_path;
            } else {
                li.textContent = "📄 " + file.display_path;
            }
            li.onclick = () => {
                document.querySelectorAll('li').forEach(el => el.classList.remove('selected'));
                li.classList.add('selected');
                selectedPath = file.full_path; 
            };
            li.ondblclick = async () => {
                if (file.entry_type == "dir") {
                    loadFiles(file.full_path);
                } else {
                    await openFile(file.full_path);
                }
            };
            fileList.appendChild(li);
        }
    } catch (e) {
        alert("Error getting files: " + e);
    }
}

pathInput.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    loadFiles(pathInput.value);
  }
});

window.addEventListener('keydown', async (event) => {
    if (event.key === 'Enter' && document.activeElement !== pathInput) {
        if (selectedPath) await openFile(selectedPath);
    }
});


function goBack() {
    if (currentHistoryIndex > 0) {
        currentHistoryIndex--;
        loadFiles(history[currentHistoryIndex], false);
    }
}

function goForward() {
    if (currentHistoryIndex < history.length - 1) {
        currentHistoryIndex++;
        loadFiles(history[currentHistoryIndex], false);
    }
}

document.getElementById('back').onclick = () => {
    goBack();
}

document.getElementById('forward').onclick = () => {
    goForward();
}

async function init() {
const urlParams = new URLSearchParams(window.location.search);
  const fileToView = urlParams.get('view');
  if (fileToView) {
    document.body.innerHTML = `<pre id="content">Loading...</pre>`;
    try {
        const content = await invoke('read_text_file', { path: fileToView });
        document.getElementById('content').textContent = content;
    } catch (e) {
        document.getElementById('content').textContent = "Error: " + e;
    }
    return;
  }
  try {
    const home = await invoke('get_home_dir');
    loadFiles(home);
  } catch (e) {
    alert(e);
    loadFiles("/");
  }
  await listen('create-dir', async (event) => {
    let dirPath = globalPath + "/" + prompt("Enter directory name:");
    try {
        await invoke('create_dir', { path: dirPath });
        loadFiles(globalPath);
    } catch (e) {
        alert("Error creating directory: " + e);
    }
  });
  await listen('create-file', async (event) => {
    let filePath = globalPath + "/" + prompt("Enter file name:");
    try {
        await invoke('create_file', { path: filePath });
        loadFiles(globalPath);
    } catch (e) {
        alert("Error creating file: " + e);
    }
  });
  await listen('open-file', async (event) => {
    if (!selectedPath) return;
    await openFile(selectedPath);
  });
  await listen('view-file', async (event) => {
    if (!selectedPath) return;
    try {
        await invoke('view_file', { path: selectedPath });
    } catch (e) {
        alert("Error viewing file: " + e);
    }
  });
  await listen('copy', async (event) => {
    if (!selectedPath) return;
    try {
        await invoke('copy', { path: selectedPath });
    } catch (e) {
        alert("Error copying file path to clipboard: " + e);
    }
  });
  await listen('delete', async (event) => {
    if (!selectedPath) return;
    try {
        await invoke('delete', { path: selectedPath });
        loadFiles(globalPath);
    } catch (e) {
        alert("Error deleting file: " + e);
    }
  });
  await listen('refresh', async (event) => {
    loadFiles(globalPath);
  });
  await listen('toggle-hidden', async (event) => {
    showHidden = !showHidden;
    loadFiles(globalPath);
  });
  await listen('go-back', async (event) => {
    goBack();
  });
  await listen('go-forward', async (event) => {
    goForward();
  });
  await listen('go-up', async (event) => {
    await goUp();
  });
}

window.addEventListener('DOMContentLoaded', init);