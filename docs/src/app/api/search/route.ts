// This route generates the static search index at build time.
// It is not called at runtime — the client uses type: "static" to load the pre-built index.
import { createSearchAPI } from "fumadocs-core/search/server";
import { source } from "@/lib/source";

export const dynamic = "force-static";

const indexes = await Promise.all(
  source.getPages().map(async (page) => {
    const data =
      "load" in page.data && typeof page.data.load === "function"
        ? await page.data.load()
        : page.data;

    const structuredData = data.structuredData ?? {
      headings: [],
      contents: [],
    };

    return {
      id: page.url,
      title: data.title ?? "Untitled",
      description: data.description ?? "",
      url: page.url,
      structuredData,
    };
  }),
);

export const { staticGET: GET } = createSearchAPI("advanced", {
  language: "english",
  indexes,
});
