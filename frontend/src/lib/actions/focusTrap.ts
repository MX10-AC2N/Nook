/**
 * focusTrap — Svelte 5 action that traps keyboard focus inside a modal/dialog.
 *
 * Usage:  <div role="dialog" use:focusTrap> ... </div>
 *
 * Behaviour:
 *  - On mount, moves focus to the first focusable element inside the node
 *    (or the node itself if it is focusable).
 *  - While open, Tab / Shift+Tab cycle within the node's focusable descendants.
 *  - On destroy, restores focus to the element that was focused before the
 *    trap mounted (standard dialog focus-return behaviour).
 *
 * Escape handling is intentionally left to the page (most Nook modals already
 * close on Escape via their own onkeydown handler).
 */
import type { Action } from 'svelte/action';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'area[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  'iframe',
  'object',
  'embed',
  'audio[controls]',
  'video[controls]',
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

export interface FocusTrapOptions {
  /** Element to focus initially; defaults to the first focusable inside the node. */
  initialFocus?: HTMLElement | null;
}

export const focusTrap: Action<HTMLElement, FocusTrapOptions | undefined> = (node, options) => {
  let opts = options;

  // Capture the element that had focus before the trap mounted, so we can
  // restore it when the dialog closes.
  const previouslyFocused =
    typeof document !== 'undefined' ? (document.activeElement as HTMLElement | null) : null;

  function getFocusable(): HTMLElement[] {
    if (typeof document === 'undefined') return [];
    return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
      (el) => el.offsetWidth > 0 || el.offsetHeight > 0 || el === document.activeElement
    );
  }

  function focusInitial() {
    const target = opts?.initialFocus ?? getFocusable()[0] ?? null;
    if (target) {
      target.focus();
    } else if (node.hasAttribute('tabindex')) {
      node.focus();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    const focusable = getFocusable();
    if (focusable.length === 0) {
      event.preventDefault();
      if (node.hasAttribute('tabindex')) node.focus();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement as HTMLElement | null;

    if (event.shiftKey) {
      if (active === first || !node.contains(active)) {
        event.preventDefault();
        last.focus();
      }
    } else {
      if (active === last || !node.contains(active)) {
        event.preventDefault();
        first.focus();
      }
    }
  }

  // Defer the initial focus until after the element is laid out, so focusable
  // children are guaranteed to be present and measurable.
  const rafId =
    typeof requestAnimationFrame !== 'undefined'
      ? requestAnimationFrame(focusInitial)
      : (focusInitial(), 0);

  node.addEventListener('keydown', handleKeydown);

  return {
    update(newOptions) {
      opts = newOptions;
    },
    destroy() {
      if (typeof cancelAnimationFrame !== 'undefined') cancelAnimationFrame(rafId);
      node.removeEventListener('keydown', handleKeydown);
      previouslyFocused?.focus?.();
    }
  };
};
