import { invoke } from "@tauri-apps/api/core";

export async function getFileFromPath(path: string): Promise<File> {
  const [content, mime]: [number[], string] = await invoke('read_file', { path });

  const bytes = new Uint8Array(content);

  const name = path.split('/').pop() || 'unknown';

  return new File([bytes], name, { type: mime });
}

export async function getFilesFromPaths(paths: string[]): Promise<File[]> {
  return Promise.all(paths.map(getFileFromPath));
}

export type UIFile = {
  id: string;
  file: File;
};

export function factorFilesForUI(newFiles: File[]): UIFile[] {
  return newFiles.map((file: File) => ({
    id: crypto.randomUUID(),
    file,
  }));
}

export async function getImageSize(file: File): Promise<{ width: number, height: number }> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      resolve({ width: img.width, height: img.height });
    };
    img.src = URL.createObjectURL(file);
  });
}
