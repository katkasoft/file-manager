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

function selectElement(li, path) {
    document.querySelectorAll('li').forEach(el => el.classList.remove('selected'));
    li.classList.add('selected');
    selectedPath = path;
    li.scrollIntoView({ block: 'nearest' });
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
        if (files.length === 0) {
            fileList.innerHTML = '<p id="empty">Folder is empty</p>';
        }
        for (const file of files) {
            if (file.display_path.startsWith('.') && !showHidden) continue;
            const li = document.createElement('li');
            li.dataset.path = file.full_path;
            li.dataset.type = file.entry_type;
            if (file.entry_type == "dir") {
                li.textContent = "📁 " + file.display_path;
            } else {
                li.textContent = "📄 " + file.display_path;
            }
            li.onclick = () => {
                selectElement(li, file.full_path);
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
    if (document.activeElement === pathInput) return;
    const items = Array.from(fileList.querySelectorAll('li'));
    const selectedIndex = items.findIndex(li => li.classList.contains('selected'));
    if (event.key === 'ArrowDown') {
        event.preventDefault();
        const nextIndex = selectedIndex < items.length - 1 ? selectedIndex + 1 : 0;
        const nextItem = items[nextIndex];
        if (nextItem) {
            selectElement(nextItem, nextItem.dataset.path);
        }
    } 
    else if (event.key === 'ArrowUp') {
        event.preventDefault();
        const prevIndex = selectedIndex > 0 ? selectedIndex - 1 : items.length - 1;
        const prevItem = items[prevIndex];
        if (prevItem) {
            selectElement(prevItem, prevItem.dataset.path);
        }
    } 
    else if (event.key === 'Enter') {
        if (selectedPath) {
            const selectedLi = items[selectedIndex];
            if (selectedLi && selectedLi.dataset.type === 'dir') {
                loadFiles(selectedPath);
            } else {
                await openFile(selectedPath);
            }
        }
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
    let inputed = prompt("Enter directory name:");
    if (inputed === null || inputed.trim() === "") return;
    let dirPath = globalPath + "/" + inputed;
    try {
        await invoke('create_dir', { path: dirPath });
        loadFiles(globalPath);
    } catch (e) {
        alert("Error creating directory: " + e);
    }
  });
  await listen('create-file', async (event) => {
    let fileName = prompt("Enter file name:");
    if (fileName === null || fileName.trim() === "") return;
    let filePath = globalPath + "/" + fileName;
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
  await listen('paste', async (event) => {
    try {
        await invoke('paste', { destDir: globalPath }); 
        loadFiles(globalPath);
    } catch (e) {
        alert("Error pasting file: " + e);
    }
  });
  await listen('cut', async (event) => {
    if (!selectedPath) return;
    try {
        await invoke('cut', { path: selectedPath });
    } catch (e) {
        alert("Error cutting file: " + e);
    }
  }); 
  await listen('rename', async (event) => {
    if (!selectedPath) return;
    let newName = prompt("Введите новое имя:");
    if (!newName || !newName.trim()) return;
    const newFullPath = globalPath.endsWith('/') ? globalPath + newName : globalPath + '/' + newName;
    try {
        await invoke('rename', { path: selectedPath, newName: newFullPath }); 
        await loadFiles(globalPath);
    } catch (e) {
        alert("Ошибка: " + e);
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