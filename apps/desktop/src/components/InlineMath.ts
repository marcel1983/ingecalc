import { mergeAttributes, Node } from "@tiptap/core";
import * as katex from "katex";
import { convertLatexToAsciiMath } from "mathlive/ssr";
import { normalizeAsciiMath } from "../lib/mathAscii";

export function latexToAscii(latex: string): string {
  try {
    return normalizeAsciiMath(convertLatexToAsciiMath(latex) || latex, latex);
  } catch {
    return latex;
  }
}

export const InlineMath = Node.create({
  name: "inlineMath",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      latex: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-latex") ?? "",
        renderHTML: (attributes) => ({ "data-latex": attributes.latex })
      },
      ascii: {
        default: "",
        parseHTML: (element) =>
          element.getAttribute("data-ascii") ??
          element.getAttribute("data-latex") ??
          "",
        renderHTML: (attributes) => ({ "data-ascii": attributes.ascii })
      }
    };
  },

  parseHTML() {
    return [{ tag: "span[data-inline-math]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-inline-math": "",
        class: "inline-math"
      })
    ];
  },

  addNodeView() {
    return ({ node, editor, getPos }) => {
      const dom = document.createElement("span");
      dom.className = "inline-math";
      dom.dataset.inlineMath = "";
      let currentNode = node;

      const render = (current: typeof node) => {
        currentNode = current;
        const latex = String(current.attrs.latex ?? "");
        dom.dataset.latex = latex;
        dom.dataset.ascii = String(current.attrs.ascii ?? "");

        try {
          dom.innerHTML = katex.renderToString(latex, {
            throwOnError: false,
            displayMode: false,
            output: "html"
          });
        } catch {
          dom.textContent = latex;
        }
      };

      const edit = () => {
        const next = globalThis.window.prompt(
          "Fórmula LaTeX (inline)",
          String(currentNode.attrs.latex ?? "")
        );
        if (!next?.trim()) {
          return;
        }

        const position = getPos();
        if (typeof position !== "number") {
          return;
        }

        const latex = next.trim();
        editor
          .chain()
          .focus()
          .command(({ tr }) => {
            tr.setNodeMarkup(position, undefined, {
              latex,
              ascii: latexToAscii(latex)
            });
            return true;
          })
          .run();
      };

      dom.addEventListener("dblclick", edit);
      render(node);

      return {
        dom,
        update: (updated) => {
          if (updated.type.name !== "inlineMath") {
            return false;
          }
          render(updated);
          return true;
        }
      };
    };
  }
});
