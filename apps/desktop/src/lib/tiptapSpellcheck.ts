import { Extension } from "@tiptap/core";
import { Plugin, PluginKey, type EditorState } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { EditorView } from "@tiptap/pm/view";
import type NSpell from "nspell";
import { isWordAccepted, isWordCorrect } from "./spellcheck";

type SpellcheckState = {
  checker: NSpell | null;
  language: string;
  accepted: Set<string>;
};

export const spellcheckPluginKey = new PluginKey<SpellcheckState>(
  "ingecalc-spellcheck"
);

const WORD_PATTERN = /[\p{L}\p{M}][\p{L}\p{M}'’\-]*/gu;

export function applySpellchecker(
  view: EditorView,
  checker: NSpell | null,
  language: string,
  accepted: Set<string> = new Set()
): void {
  view.dispatch(
    view.state.tr.setMeta(spellcheckPluginKey, {
      checker,
      language,
      accepted
    })
  );
}

function buildDecorations(state: EditorState): DecorationSet | null {
  const spellcheck = spellcheckPluginKey.getState(state);
  if (!spellcheck?.checker) {
    return null;
  }
  const checker = spellcheck.checker;

  const decorations: Decoration[] = [];
  state.doc.descendants((node, position) => {
    if (!node.isText || !node.text) {
      return;
    }
    if (node.marks.some((mark) => mark.type.name === "code")) {
      return;
    }
    if (state.doc.resolve(position).parent.type.name === "codeBlock") {
      return;
    }

    for (const match of node.text.matchAll(WORD_PATTERN)) {
      const word = match[0];
      if (word.length < 2) {
        continue;
      }
      if (isWordAccepted(word, spellcheck.accepted)) {
        continue;
      }
      if (isWordCorrect(checker, word)) {
        continue;
      }

      const from = position + (match.index ?? 0);
      decorations.push(
        Decoration.inline(from, from + word.length, {
          class: "spell-error",
          "data-word": word
        })
      );
    }
  });

  return DecorationSet.create(state.doc, decorations);
}

export const SpellcheckExtension = Extension.create({
  name: "ingecalcSpellcheck",

  addProseMirrorPlugins() {
    return [
      new Plugin<SpellcheckState>({
        key: spellcheckPluginKey,
        state: {
          init: () => ({
            checker: null,
            language: "es",
            accepted: new Set<string>()
          }),
          apply: (transaction, previous) => {
            const meta = transaction.getMeta(spellcheckPluginKey) as
              | Partial<SpellcheckState>
              | undefined;
            return meta ? { ...previous, ...meta } : previous;
          }
        },
        props: {
          decorations(state) {
            return buildDecorations(state);
          }
        }
      })
    ];
  }
});
