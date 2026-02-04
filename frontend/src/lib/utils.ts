import { type ClassValue, clsx } from 'clsx';
// import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  // TODO: Re-enable twMerge after migration to tailwind v4
  // Doesn't support de-duplicating custom classes, eg text-brand and text-base
  // return twMerge(clsx(inputs));
  return clsx(inputs);
}

export function formatFileSize(bytes: number | null | undefined): string {
  if (!bytes) return '';
  const num = bytes;
  if (num < 1024) return `${num} B`;
  if (num < 1024 * 1024) return `${(num / 1024).toFixed(1)} KB`;
  return `${(num / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Copy text to clipboard. Uses the Clipboard API when available (HTTPS),
 * falls back to execCommand('copy') for HTTP contexts (e.g. Tailscale).
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Clipboard API can throw even when present (e.g. permissions denied).
      // Fall through to legacy fallback.
    }
  }

  // Legacy fallback for non-secure contexts (HTTP)
  try {
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(textarea);
    return ok;
  } catch {
    return false;
  }
}
