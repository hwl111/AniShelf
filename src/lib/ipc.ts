import { invoke } from "@tauri-apps/api/core";

export class IpcError extends Error {
  readonly command: string;

  constructor(command: string, message: string) {
    super(message);
    this.name = "IpcError";
    this.command = command;
  }
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return "未知错误";
}

/**
 * Typed wrapper around Tauri's invoke. Normalizes backend errors into IpcError
 * so UI code can show a readable message without silently swallowing failures.
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const message = toErrorMessage(error);
    console.error(`[AniShelf] IPC command failed: ${command}`, error);
    throw new IpcError(command, message);
  }
}
