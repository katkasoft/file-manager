const { invoke } = window.__TAURI__.core;
const fileList = document.getElementById('file-list');
const pathInput = document.getElementById('path');
let globalPath = "";

document.getElementById('back').onclick = async () => {
    if (globalPath === "/") return;
    try {
        const parentPath = await invoke('get_parent_path', { path: globalPath });
        loadFiles(parentPath);
    } catch (e) {
        alert("Error going to parent directory: " + e);
    }
}

async function loadFiles(path) {
    globalPath = path;
    pathInput.value = path;
    try {
        const files = await invoke('get_files', { path: path });
        fileList.innerHTML = '';
        for (const file of files) {
            if (file.display_path.startsWith('.')) continue;
            const li = document.createElement('li');
            if (file.entry_type == "dir") {
                li.textContent = "📁 " + file.display_path;
            } else {
                li.textContent = "📄 " + file.display_path;
            }
            li.onclick = async () => {
                if (file.entry_type == "dir") {
                    loadFiles(file.full_path);
                } else {
                    try {
                        await invoke('open_file', { path: file.full_path });
                    } catch (e) {
                        alert("Error opening file: " + e);
                    }
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

async function init() {
  try {
    const home = await invoke('get_home_dir');
    loadFiles(home);
  } catch (e) {
    alert(e);
    loadFiles("/");
  }
}

window.addEventListener('DOMContentLoaded', init);
