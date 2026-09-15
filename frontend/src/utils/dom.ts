export function requireElement(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (el === null) {
    throw new Error(`Required element id ${id} was missing from DOM`);
  }

  return el;
}

export function createHtmlElement(
  tag: string,
  className?: string,
  text?: string,
): HTMLElement {
  const node = document.createElement(tag);

  if (className) {
    node.className = className;
  }

  if (text) {
    node.textContent = text;
  }

  return node;
}
