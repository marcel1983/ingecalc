import type { PageSetup, Settings } from "../types/document";

export const DEFAULT_PAGE_SETUP: PageSetup = {
  marginTop: 48,
  marginRight: 48,
  marginBottom: 48,
  marginLeft: 48,
  header: "",
  footer: ""
};

export function getPageSetup(settings?: Settings): PageSetup {
  return settings?.page ?? DEFAULT_PAGE_SETUP;
}

export function formatPageText(text: string, pageNumber: number): string {
  return text.replace(/\{page\}/gi, String(pageNumber));
}
