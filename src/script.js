const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const fileList = document.getElementById('file-list');
const pathInput = document.getElementById('path');
let globalPath = "";
let selectedPath = "";

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
            li.onclick = () => {
                document.querySelectorAll('li').forEach(el => el.classList.remove('selected'));
                li.classList.add('selected');
                selectedPath = file.full_path; 
            };
            li.ondblclick = async () => {
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
  await listen('delete', async (event) => {
    if (!selectedPath) return;
    try {
        await invoke('delete', { path: selectedPath });
        loadFiles(globalPath);
    } catch (e) {
        alert("Error deleting file: " + e);
    }
  });
}

window.addEventListener('DOMContentLoaded', init);