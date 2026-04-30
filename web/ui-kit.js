function joinClassNames(...names) {
  return names.filter(Boolean).join(" ");
}

export function createElement(tagName, options = {}) {
  const element = document.createElement(tagName);
  if (options.id) {
    element.id = options.id;
  }
  if (options.className) {
    element.className = options.className;
  }
  if (options.text !== undefined && options.text !== null) {
    element.textContent = String(options.text);
  }
  if (options.attrs) {
    for (const [name, value] of Object.entries(options.attrs)) {
      if (value === false || value === null || value === undefined) {
        continue;
      }
      element.setAttribute(name, value === true ? "" : String(value));
    }
  }
  if (options.dataset) {
    for (const [name, value] of Object.entries(options.dataset)) {
      if (value === null || value === undefined) {
        continue;
      }
      element.dataset[name] = String(value);
    }
  }
  return element;
}

export function appendChildren(parent, ...children) {
  for (const child of children.flat()) {
    if (child === null || child === undefined || child === false) {
      continue;
    }
    if (child instanceof Node) {
      parent.appendChild(child);
    } else {
      parent.appendChild(document.createTextNode(String(child)));
    }
  }
  return parent;
}

export function uiShell({ centered = false, className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-shell", centered && "ui-shell--centered", className),
  });
}

export function uiCard({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-card", className),
  });
}

export function uiHeader({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-header", className),
  });
}

export function uiHeaderCopy({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-header-copy", className),
  });
}

export function uiTitle(text, { level = 2, className = "" } = {}) {
  const tagName = `h${Math.min(6, Math.max(1, level))}`;
  return createElement(tagName, {
    className: joinClassNames("ui-title", className),
    text,
  });
}

export function uiCopy(text, { tone = "body", tag = "p", className = "" } = {}) {
  if (!text) {
    return null;
  }
  const toneClass = {
    body: "ui-copy",
    muted: "ui-copy ui-copy--muted",
    note: "ui-copy ui-copy--note",
    error: "ui-copy ui-copy--error",
  }[tone] || "ui-copy";
  return createElement(tag, {
    className: joinClassNames(toneClass, className),
    text,
  });
}

export function uiChip(text, { tone = "default", className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-chip", tone !== "default" && `ui-chip--${tone}`, className),
    text,
  });
}

export function uiChipRow({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-chip-row", className),
  });
}

export function uiStack({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-stack", className),
  });
}

export function uiSection({ className = "" } = {}) {
  return createElement("section", {
    className: joinClassNames("ui-section", className),
  });
}

export function uiToggleRow({ className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-toggle-row", className),
  });
}

export function uiButtonRow({ grow = false, className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-button-row", grow && "ui-button-row--grow", className),
  });
}

export function uiActions({ grow = false, className = "" } = {}) {
  return createElement("div", {
    className: joinClassNames("ui-actions", grow && "ui-actions--grow", className),
  });
}

export function uiPanel(title, { className = "", bodyClassName = "" } = {}) {
  const root = createElement("div", {
    className: joinClassNames("ui-panel", className),
  });
  if (title) {
    root.appendChild(
      createElement("h3", {
        className: "ui-panel-title",
        text: title,
      }),
    );
  }
  const body = createElement("div", {
    className: joinClassNames("ui-panel-body", bodyClassName),
  });
  root.appendChild(body);
  return { root, body };
}

export function uiButton(label = "", options = {}) {
  const element = createElement("button", {
    className: joinClassNames(
      "ui-button",
      options.variant ? `ui-button--${options.variant}` : "ui-button--secondary",
      options.className,
    ),
    attrs: {
      type: "button",
      ...options.attrs,
    },
    dataset: {
      action: options.action,
      ...options.dataset,
    },
  });
  if (options.disabled) {
    element.disabled = true;
  }
  if (label) {
    element.textContent = label;
  }
  if (options.children) {
    appendChildren(element, options.children);
  }
  return element;
}
